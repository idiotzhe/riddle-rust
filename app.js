import { Hono } from 'hono';
import { serveStatic } from 'hono/bun';
import { getCookie, setCookie, deleteCookie } from 'hono/cookie';
import { cors } from 'hono/cors';
import { serve } from '@hono/node-server';
import { Server } from 'socket.io';
import nunjucks from 'nunjucks';
import path from 'path';
import fs from 'fs';
import { v4 as uuidv4 } from 'uuid';
import moment from 'moment';
import * as xlsx from 'xlsx';
import db from './models.js';

const app = new Hono();
const port = 6000;

console.log('Current directory:', process.cwd());
console.log('Script directory:', import.meta.dir);

// Nunjucks configuration
const templateDir = path.join(import.meta.dir, 'template');
console.log('Template directory:', templateDir);
const env = nunjucks.configure(templateDir, {
    autoescape: true,
    watch: true
});

// Custom filter for Activity time range display
env.addFilter('get_time_range_display', function(activity) {
    if (!activity || !activity.start_time || !activity.end_time) return '';
    const start = moment(activity.start_time);
    const end = moment(activity.end_time);
    
    if (start.isSame(end, 'day')) {
        return `<p>${start.format('YYYY/MM/DD')}</p><p>${start.format('HH:mm')}~${end.format('HH:mm')}</p>`;
    } else {
        return `<p>${start.format('MM/DD HH:mm')}~</p><p> ${end.format('MM/DD HH:mm')}</p>`;
    }
});

const render = (template, data = {}) => {
    return env.render(template, data);
};

// Middleware
app.use('*', cors());

// Helper function to map SQL objects to dict-like objects
const toDict = (obj, type) => {
    if (!obj) return null;
    const result = { ...obj };
    if (obj.is_solved !== undefined) {
        result.is_solved = !!obj.is_solved;
    }
    if (type === 'user') {
        result.register_time = moment(obj.register_time).format('YYYY-MM-DD HH:mm:ss');
    } else if (type === 'riddle') {
        try {
            result.options = JSON.parse(obj.options_json || '[]');
        } catch (e) {
            result.options = [];
        }
        result.add_time = moment(obj.add_time).format('YYYY-MM-DD HH:mm:ss');
    } else if (type === 'activity') {
        result.start_time = moment(obj.start_time).format('YYYY-MM-DD HH:mm:ss');
        result.end_time = moment(obj.end_time).format('YYYY-MM-DD HH:mm:ss');
        result.get_time_range_display = function() {
            const start = moment(this.start_time);
            const end = moment(this.end_time);
            if (start.isSame(end, 'day')) {
                return `<p>${start.format('YYYY/MM/DD')}</p><p>${start.format('HH:mm')}~${end.format('HH:mm')}</p>`;
            } else {
                return `<p>${start.format('MM/DD HH:mm')}~</p><p> ${end.format('MM/DD HH:mm')}</p>`;
            }
        };
    } else if (type === 'record') {
        result.solve_time = moment(obj.solve_time).format('YYYY-MM-DD HH:mm:ss');
    }
    return result;
};

// Auth middleware replacement
const getUserId = (c) => {
    return getCookie(c, 'user_id');
};

const loginRequired = async (c, next) => {
    const userId = getUserId(c);
    if (!userId) {
        return c.json({ error: '未登录', code: 'NOT_LOGGED_IN' }, 401);
    }
    const user = db.prepare('SELECT * FROM users WHERE id = ?').get(userId);
    if (!user) {
        deleteCookie(c, 'user_id');
        return c.json({ error: '用户不存在', code: 'USER_NOT_FOUND' }, 401);
    }
    c.set('currentUser', user);
    await next();
};

// ============================
// Client APIs
// ============================

app.get('/lantern', (c) => {
    let activity = toDict(db.prepare('SELECT * FROM activities LIMIT 1').get(), 'activity') || {};
    return c.html(render('index.html', { activity }));
});

app.get('/frontend/index', (c) => {
    let activity = toDict(db.prepare('SELECT * FROM activities LIMIT 1').get(), 'activity') || {};
    return c.html(render('frontend/index.html', { activity }));
});

app.get('/frontend/riddles', (c) => {
    const page = parseInt(c.req.query('page')) || 1;
    const pageSize = parseInt(c.req.query('pageSize')) || 1;
    const excludeIds = c.req.query('exclude_ids') ? c.req.query('exclude_ids').split(',').map(id => parseInt(id)) : [];
    const offset = (page - 1) * pageSize;

    let query = `
        SELECT r.*, u.username as solver_name, u.avatar as solver_avatar 
        FROM riddles r 
        LEFT JOIN users u ON r.solver_id = u.id 
        WHERE r.is_solved = 0 
    `;
    let params = [];
    if (excludeIds.length > 0) {
        query += ` AND r.id NOT IN (${excludeIds.map(() => '?').join(',')})`;
        params.push(...excludeIds);
    }
    query += ' LIMIT ? OFFSET ?';
    params.push(pageSize, offset);

    const items = db.prepare(query).all(...params);

    if (items.length === 0) {
        return c.json({ code: 404, message: "没有数据" });
    }

    return c.json({
        code: 200,
        message: "success",
        data: items.map(item => toDict(item, 'riddle'))
    });
});

app.get('/frontend/riddle/:id', (c) => {
    const riddle = db.prepare(`
        SELECT r.*, u.username as solver_name, u.avatar as solver_avatar 
        FROM riddles r 
        LEFT JOIN users u ON r.solver_id = u.id 
        WHERE r.id = ?
    `).get(c.req.param('id'));

    if (riddle) {
        return c.json({ data: toDict(riddle, 'riddle'), code: 200 });
    }
    return c.json({ code: 400 });
});

app.get('/q', (c) => {
    const riddleId = c.req.query('r_id');
    const riddle = toDict(db.prepare('SELECT * FROM riddles WHERE id = ?').get(riddleId), 'riddle');
    const userId = getUserId(c);
    
    if (!userId) {
        const activity = toDict(db.prepare('SELECT * FROM activities LIMIT 1').get(), 'activity') || {};
        return c.html(render('index.html', { riddle_id: riddleId, activity }));
    }
    
    const user = db.prepare('SELECT * FROM users WHERE id = ?').get(userId);
    if (!user) {
        const activity = toDict(db.prepare('SELECT * FROM activities LIMIT 1').get(), 'activity') || {};
        return c.html(render('index.html', { riddle_id: riddleId, activity }));
    }
    
    return c.html(render('question.html', { riddle }));
});

app.post('/login', async (c) => {
    const body = await c.req.parseBody();
    const username = body['username'];
    const file = body['file'];

    if (!username) {
        return c.json({ error: '昵称不能为空' }, 400);
    }

    let avatar_path = "";
    if (file && file instanceof File) {
        const today = moment().format('YYYY/MM/DD');
        const uploadDir = path.join(import.meta.dir, 'template', 'avatar', today);
        if (!fs.existsSync(uploadDir)) {
            fs.mkdirSync(uploadDir, { recursive: true });
        }
        const ext = path.extname(file.name).toLowerCase();
        const filename = uuidv4().replace(/-/g, '') + ext;
        const filePath = path.join(uploadDir, filename);
        
        const bytes = await file.arrayBuffer();
        await Bun.write(filePath, bytes);
        
        avatar_path = "/avatar/" + today + "/" + filename;
    }

    const user_code = uuidv4().substring(0, 8).toUpperCase();
    const dummy_token = uuidv4();

    const now = moment().format('YYYY-MM-DD HH:mm:ss');
    const info = db.prepare('INSERT INTO users (username, avatar, user_code, token, register_time) VALUES (?, ?, ?, ?, ?)').run(username, avatar_path, user_code, dummy_token, now);
    const user = db.prepare('SELECT * FROM users WHERE id = ?').get(info.lastInsertRowid);

    setCookie(c, 'user_id', user.id.toString(), {
        maxAge: 31 * 24 * 60 * 60, // 31 days in seconds
        path: '/',
    });

    return c.json({
        msg: '登录成功',
        user_info: toDict(user, 'user')
    });
});

app.all('/logout', (c) => {
    deleteCookie(c, 'user_id');
    return c.json({ msg: '已退出登录' });
});

app.post('/guess', loginRequired, async (c) => {
    // Handle both JSON and Form data
    let body;
    const contentType = c.req.header('Content-Type');
    if (contentType && contentType.includes('application/json')) {
        body = await c.req.json();
    } else {
        body = await c.req.parseBody();
    }

    const { riddle_id, answer } = body;
    const user_answer = (answer || '').trim();
    const currentUser = c.get('currentUser');

    if (!riddle_id || !user_answer) {
        return c.json({ msg: '参数不全', code: 400 });
    }

    const activity = db.prepare('SELECT * FROM activities LIMIT 1').get();
    if (activity) {
        const now = moment();
        if (activity.start_time && now.isBefore(moment(activity.start_time))) {
            return c.json({ msg: '活动尚未开始', code: 400 });
        }
        if (activity.end_time && now.isAfter(moment(activity.end_time))) {
            return c.json({ msg: '活动已经结束', code: 400 });
        }
    }

    const riddle = db.prepare('SELECT r.*, u.username as solver_name FROM riddles r LEFT JOIN users u ON r.solver_id = u.id WHERE r.id = ?').get(riddle_id);
    if (!riddle) {
        return c.json({ msg: '题目不存在', code: 404 });
    }

    if (riddle.is_solved) {
        return c.json({
            success: false,
            msg: `太可惜了，这道题已经被 ${riddle.solver_name || "别人"} 抢先猜中了！`,
            code: 400
        });
    }

    const existingRecord = db.prepare('SELECT * FROM guess_records WHERE user_id = ? AND riddle_id = ?').get(currentUser.id, riddle_id);
    if (existingRecord) {
        return c.json({ success: false, msg: '你已经猜过该题了！', code: 400 });
    }

    const now_time = moment().format('YYYY-MM-DD HH:mm:ss');
    if (user_answer.toLowerCase() === riddle.answer.toLowerCase()) {
        const recheck = db.prepare('SELECT is_solved FROM riddles WHERE id = ?').get(riddle_id);
        if (recheck.is_solved) {
            return c.json({ success: false, msg: '手慢了，已被抢答！', code: 400 });
        }

        db.prepare('UPDATE riddles SET is_solved = 1, solver_id = ? WHERE id = ?').run(currentUser.id, riddle_id);
        db.prepare('INSERT INTO guess_records (user_id, riddle_id, is_solved, solve_time) VALUES (?, ?, 1, ?)').run(currentUser.id, riddle_id, now_time);

        // Broadcast to all clients
        if (global.io) {
            global.io.emit('riddle_solved', {
                riddle_id: riddle_id,
                solver_name: currentUser.username,
                solver_avatar: currentUser.avatar
            });
        }

        return c.json({ success: true, msg: '恭喜你！抢答成功！', code: 200 });
    } else {
        db.prepare('INSERT INTO guess_records (user_id, riddle_id, is_solved, solve_time) VALUES (?, ?, 0, ?)').run(currentUser.id, riddle_id, now_time);
        return c.json({ success: false, msg: '答案不对，请再接再厉！' });
    }
});

app.get('/my/records', loginRequired, (c) => {
    const currentUser = c.get('currentUser');
    const records = db.prepare(`
        SELECT gr.*, r.question as riddle_question, r.answer as riddle_answer 
        FROM guess_records gr
        JOIN riddles r ON gr.riddle_id = r.id
        WHERE gr.user_id = ? 
        ORDER BY gr.solve_time DESC
    `).all(currentUser.id);
    
    return c.json(records.map(r => toDict(r, 'record')));
});

// ============================
// Admin APIs (prefix /pro-api)
// ============================
const adminApi = new Hono();

adminApi.get('/index', (c) => {
    return c.html(render('admin/index.html'));
});

adminApi.get('/users', (c) => {
    const page = parseInt(c.req.query('page')) || 1;
    const pageSize = parseInt(c.req.query('pageSize')) || 10;
    const keyword = c.req.query('keyword');
    const offset = (page - 1) * pageSize;

    let query = 'SELECT * FROM users';
    let countQuery = 'SELECT COUNT(*) as total FROM users';
    let params = [];

    if (keyword) {
        query += ' WHERE username LIKE ?';
        countQuery += ' WHERE username LIKE ?';
        params.push(`%${keyword}%`);
    }

    const total = db.prepare(countQuery).get(...params).total;
    query += ' LIMIT ? OFFSET ?';
    const items = db.prepare(query).all(...params, pageSize, offset);

    return c.json({
        code: 200,
        message: "success",
        data: {
            total,
            page,
            totalPages: Math.ceil(total / pageSize),
            list: items.map(u => toDict(u, 'user'))
        }
    });
});

adminApi.delete('/user/:id', (c) => {
    const info = db.prepare('DELETE FROM users WHERE id = ?').run(c.req.param('id'));
    if (info.changes > 0) {
        return c.json({ msg: '删除成功' });
    }
    return c.json({ error: '用户不存在' }, 404);
});

adminApi.get('/riddles', (c) => {
    const page = parseInt(c.req.query('page')) || 1;
    const pageSize = parseInt(c.req.query('pageSize')) || 10;
    const keyword = c.req.query('keyword');
    const offset = (page - 1) * pageSize;

    let query = 'SELECT r.*, u.username as solver_name, u.avatar as solver_avatar FROM riddles r LEFT JOIN users u ON r.solver_id = u.id';
    let countQuery = 'SELECT COUNT(*) as total FROM riddles';
    let params = [];

    if (keyword) {
        query += ' WHERE r.question LIKE ?';
        countQuery += ' WHERE question LIKE ?';
        params.push(`%${keyword}%`);
    }

    const total = db.prepare(countQuery).get(...params).total;
    query += ' LIMIT ? OFFSET ?';
    const items = db.prepare(query).all(...params, pageSize, offset);

    return c.json({
        code: 200,
        message: "success",
        data: {
            total,
            page,
            totalPages: Math.ceil(total / pageSize),
            list: items.map(r => toDict(r, 'riddle'))
        }
    });
});

adminApi.post('/riddles', async (c) => {
    let data;
    const contentType = c.req.header('Content-Type');
    if (contentType && contentType.includes('application/json')) {
        data = await c.req.json();
    } else {
        data = await c.req.parseBody();
    }

    if (data.id) {
        const riddle = db.prepare('SELECT * FROM riddles WHERE id = ?').get(data.id);
        if (!riddle) return c.json({ error: '不存在' }, 404);
        
        const question = data.question || riddle.question;
        const answer = data.answer || riddle.answer;
        const remark = data.remark !== undefined ? data.remark : riddle.remark;
        const options_json = data.options ? JSON.stringify(data.options) : riddle.options_json;
        
        let is_solved = riddle.is_solved;
        let solver_id = riddle.solver_id;
        if (data.reset_status === true || data.reset_status === 'true') {
            is_solved = 0;
            solver_id = null;
        }

        db.prepare('UPDATE riddles SET question = ?, answer = ?, remark = ?, options_json = ?, is_solved = ?, solver_id = ? WHERE id = ?')
          .run(question, answer, remark, options_json, is_solved, solver_id, data.id);
        
        const updated = db.prepare('SELECT r.*, u.username as solver_name FROM riddles r LEFT JOIN users u ON r.solver_id = u.id WHERE r.id = ?').get(data.id);
        return c.json(toDict(updated, 'riddle'));
    } else {
        const options_json = JSON.stringify(data.options || []);
        const now_time = moment().format('YYYY-MM-DD HH:mm:ss');
        const info = db.prepare('INSERT INTO riddles (question, answer, remark, options_json, add_time) VALUES (?, ?, ?, ?, ?)')
          .run(data.question, data.answer, data.remark || '', options_json, now_time);
        
        const inserted = db.prepare('SELECT * FROM riddles WHERE id = ?').get(info.lastInsertRowid);
        return c.json(toDict(inserted, 'riddle'));
    }
});

adminApi.post('/riddles/import', async (c) => {
    const body = await c.req.parseBody();
    const file = body['file'];

    if (!file && !Array.isArray(body)) {
        return c.json({ code: 400, msg: '未上传文件或数据格式错误' }, 400);
    }

    if (file && file instanceof File) {
        try {
            const buffer = await file.arrayBuffer();
            const workbook = xlsx.read(buffer);
            const sheetName = workbook.SheetNames[0];
            const worksheet = workbook.Sheets[sheetName];
            const data = xlsx.utils.sheet_to_json(worksheet);

            let successCount = 0;
            const now_time = moment().format('YYYY-MM-DD HH:mm:ss');
            const insert = db.prepare('INSERT INTO riddles (question, answer, remark, options_json, add_time) VALUES (?, ?, ?, ?, ?)');
            
            for (const row of data) {
                const question = row['灯谜题目'];
                const answer = row['正确答案'];
                const remark = row['描述'] || '';
                if (!question || !answer) continue;

                const options = [];
                Object.keys(row).forEach(key => {
                    if (key.includes('选项') && row[key] !== undefined) {
                        options.push(String(row[key]).trim());
                    }
                });

                // Shuffle options
                for (let i = options.length - 1; i > 0; i--) {
                    const j = Math.floor(Math.random() * (i + 1));
                    [options[i], options[j]] = [options[j], options[i]];
                }

                insert.run(String(question).trim(), String(answer).trim(), remark, JSON.stringify(options), now_time);
                successCount++;
            }
            
            return c.json({ code: 200, msg: `灯谜导入成功: 共 ${successCount} 条` });
        } catch (e) {
            return c.json({ code: 500, msg: `服务器内部错误: ${e.message}` }, 500);
        }
    } else {
        // Handle JSON bulk import
        let data = body;
        if (typeof body === 'string') {
            try { data = JSON.parse(body); } catch(e) {}
        }
        if (Array.isArray(data)) {
            const now_time = moment().format('YYYY-MM-DD HH:mm:ss');
            const insert = db.prepare('INSERT INTO riddles (question, answer, remark, options_json, add_time) VALUES (?, ?, ?, ?, ?)');
            data.forEach(item => {
                insert.run(item.question, item.answer, item.remark || '', JSON.stringify(item.options || []), now_time);
            });
            return c.json({ msg: '导入成功' });
        }
        return c.json({ code: 400, msg: '数据格式错误' }, 400);
    }
});

adminApi.delete('/riddle/:id', (c) => {
    db.prepare('DELETE FROM guess_records WHERE riddle_id = ?').run(c.req.param('id'));
    const info = db.prepare('DELETE FROM riddles WHERE id = ?').run(c.req.param('id'));
    if (info.changes > 0) {
        return c.json({ msg: '删除成功' });
    }
    return c.json({ error: '不存在' }, 404);
});

adminApi.get('/leaderboard', (c) => {
    const page = parseInt(c.req.query('page')) || 1;
    const pageSize = parseInt(c.req.query('pageSize')) || 10;
    const keyword = c.req.query('keyword');
    const offset = (page - 1) * pageSize;

    let query = `
        SELECT gr.*, u.username as user_name, r.question as riddle_question, r.answer as riddle_answer 
        FROM guess_records gr
        JOIN users u ON gr.user_id = u.id
        JOIN riddles r ON gr.riddle_id = r.id
        WHERE gr.is_solved = 1
    `;
    let countQuery = `
        SELECT COUNT(*) as total 
        FROM guess_records gr
        JOIN users u ON gr.user_id = u.id
        WHERE gr.is_solved = 1
    `;
    let params = [];

    if (keyword) {
        query += ' AND u.username LIKE ?';
        countQuery += ' AND u.username LIKE ?';
        params.push(`%${keyword}%`);
    }

    const total = db.prepare(countQuery).get(...params).total;
    query += ' ORDER BY gr.solve_time DESC LIMIT ? OFFSET ?';
    const items = db.prepare(query).all(...params, pageSize, offset);

    return c.json({
        code: 200,
        message: "success",
        data: {
            total,
            page,
            totalPages: Math.ceil(total / pageSize),
            list: items.map(r => toDict(r, 'record'))
        }
    });
});

adminApi.get('/activity', (c) => {
    let act = db.prepare('SELECT * FROM activities LIMIT 1').get();
    if (!act) {
        db.prepare("INSERT INTO activities (name, start_time, end_time) VALUES ('元宵猜灯谜', datetime('now'), datetime('now', '+1 day'))").run();
        act = db.prepare('SELECT * FROM activities LIMIT 1').get();
    }
    return c.json(toDict(act, 'activity'));
});

adminApi.post('/activity', async (c) => {
    let data;
    const contentType = c.req.header('Content-Type');
    if (contentType && contentType.includes('application/json')) {
        data = await c.req.json();
    } else {
        data = await c.req.parseBody();
    }

    const { name, start_time, end_time } = data;
    let act = db.prepare('SELECT * FROM activities LIMIT 1').get();
    if (!act) {
        db.prepare('INSERT INTO activities (name, start_time, end_time) VALUES (?, ?, ?)')
          .run(name || '元宵猜灯谜', start_time, end_time);
    } else {
        db.prepare('UPDATE activities SET name = ?, start_time = ?, end_time = ? WHERE id = ?')
          .run(name || '元宵猜灯谜', start_time, end_time, act.id);
    }
    const updated = db.prepare('SELECT * FROM activities LIMIT 1').get();
    return c.json(toDict(updated, 'activity'));
});

app.route('/pro-api', adminApi);

// Serve static files from 'template' folder
app.use('/*', serveStatic({ root: './template' }));

const server = serve({
  fetch: app.fetch,
  port: port
}, (info) => {
  console.log(`Server running at http://localhost:${info.port}`);
});

// Socket.io initialization
const io = new Server(server, {
    cors: {
        origin: "*",
        methods: ["GET", "POST"]
    }
});

io.on('connection', (socket) => {
    console.log('A client connected');
});

global.io = io;

export { io };
