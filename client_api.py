from flask import Blueprint, request, jsonify, g, render_template, session
from web_model import db, User, Riddle, GuessRecord, Activity
import uuid
from datetime import datetime
from functools import wraps
import socket
import os


# 配置文件上传
class Config:
    # ... 其他配置
    UPLOAD_FOLDER = 'template/avatar'  # 头像上传目录
    MAX_CONTENT_LENGTH = 16 * 1024 * 1024  # 限制16MB
    ALLOWED_EXTENSIONS = {'png', 'jpg', 'jpeg', 'gif', 'webp'}

def allowed_file(filename):
    """检查文件扩展名是否允许"""
    return '.' in filename and filename.rsplit('.', 1)[1].lower() in Config.ALLOWED_EXTENSIONS



client_bp = Blueprint('client', __name__, url_prefix='')


@client_bp.route('/lantern')
def index():
    # 把开始结束时间变成
    return render_template('index.html',activity=Activity.query.first())


@client_bp.route('frontend/index')
def frontend():
    return render_template('frontend/index.html',activity=Activity.query.first())

# ----------------------------
# 获取一个灯谜
# ----------------------------
@client_bp.route('frontend/riddles', methods=['GET'])
def get_frontend_riddles():
    # pageSize: 每页大小 (默认 10)
    page = request.args.get('page', 1, type=int)
    page_size = request.args.get('pageSize', 1, type=int)
    # 2. 执行分页查询
    # SQLAlchemy 的 paginate 方法中参数依然是 per_page，我们将 page_size 传进去
    pagination = Riddle.query.filter_by(is_solved=False).paginate(page=page, per_page=page_size, error_out=False)
    # 判断是否有数据
    if not pagination.items:
        return jsonify({
            "code": 404,
            "message": "没有数据"
        })
    return jsonify({
        "code": 200,
        "message": "success",
        "data":  pagination.items[0].to_dict()
    })

# ----------------------------
# 获取一个灯谜
# ----------------------------
@client_bp.route('frontend/riddle/<int:riddle_id>', methods=['GET'])
def get_frontend_riddle_info(riddle_id):
    riddle = Riddle.query.get(riddle_id)
    if riddle:
        return jsonify({'data':riddle.to_dict(),'code':200})
    return jsonify({"code",400})


@client_bp.route('/q')
def question():
    riddle = Riddle.query.get(request.args.get("r_id"))
    # 判断是否存在user_id
    if not session.get('user_id'):
        return render_template('index.html',riddle_id=riddle.id,activity=Activity.query.first())
    user=User.query.get(session['user_id'])
    if user is None:
        return render_template('index.html',riddle_id=riddle.id,activity=Activity.query.first())
    return render_template('question.html', riddle=riddle)



# ============================
# 辅助装饰器: 验证 Session
# ============================
def login_required(f):
    @wraps(f)
    def decorated_function(*args, **kwargs):
        # 1. 从 Session 中获取 user_id
        user_id = session.get('user_id')

        if not user_id:
            return jsonify({'error': '未登录', 'code': 'NOT_LOGGED_IN'}), 401

        # 2. 从数据库获取用户对象
        user = User.query.get(user_id)

        if not user:
            # 这种情况可能是用户ID存在session里，但数据库把人删了
            session.clear()
            return jsonify({'error': '用户不存在', 'code': 'USER_NOT_FOUND'}), 401

        # 3. 存入全局变量 g，供后续视图使用
        g.current_user = user
        return f(*args, **kwargs)

    return decorated_function


# ============================
# 5. 客户端接口/登录
# ============================
@client_bp.route('/login', methods=['POST'])
def login():
    """
    用户输入昵称/上传头像即可注册/登录
    改用 Session 维持状态
    """
    data = request.form
    username = data.get('username')
    if not username:
        return jsonify({'error': '昵称不能为空'}), 400

    file = request.files['file']
    avatar_path = ""
    if file and file.filename:
        if not allowed_file(file.filename):
            return jsonify({'error': '不支持的文件格式'}), 400

        # 生成保存路径：avatar/2024/01/01/随机名.png
        today = datetime.now()
        date_path = today.strftime('%Y/%m/%d')

        # 确保目录存在
        upload_dir = os.path.join(Config.UPLOAD_FOLDER, date_path)
        os.makedirs(upload_dir, exist_ok=True)

        # 生成文件名并保存
        file_ext = file.filename.rsplit('.', 1)[1].lower()
        new_filename = f"{uuid.uuid4().hex}.{file_ext}"
        file_path = os.path.join(upload_dir, new_filename)
        file.save(file_path)

        # 数据库存储的相对路径
        avatar_path ="/avatar/"+ os.path.join(date_path, new_filename).replace('\\', '/')

    # --- 注册新用户 ---
    # 生成唯一编码 (8位随机码)
    user_code = str(uuid.uuid4())[:8].upper()

    # 注意：既然用了Session，User模型里的 token 字段其实就不需要了。
    # 如果数据库还没删该字段，这里可以填个占位符，或者修改 Model 删掉该字段。
    # 这里假设你还没改 Model，给个随机值填充。
    dummy_token = str(uuid.uuid4())

    user = User(
        username=username,
        avatar=avatar_path,
        user_code=user_code,
        token=dummy_token  # 如果你的 User 模型里删除了 token 列，把这行去掉
    )
    db.session.add(user)
    db.session.commit()

    # ==========================
    # 核心修改：设置 Session
    # ==========================
    session.permanent = True  # 设置为永久 Session (默认有效期通常是31天，需配合配置)
    session['user_id'] = user.id
    return jsonify({
        'msg': '登录成功',
        'user_info': user.to_dict()
    })

# ============================
# 新增：登出接口
# ============================
@client_bp.route('/logout', methods=['POST', 'GET'])
def logout():
    session.clear()  # 清空 Session
    return jsonify({'msg': '已退出登录'})


# ============================
# 6. 猜灯谜接口
# ============================
@client_bp.route('/guess', methods=['POST'])
@login_required  # 这里现在验证的是 Session
def guess_riddle():
    """
    抢答模式：一个灯谜只能被全场猜中一次。
    """
    riddle_id = request.form.get('riddle_id')
    user_answer = request.form.get('answer', '').strip()

    if not riddle_id or not user_answer:
        return jsonify({'msg': '参数不全', 'code': 400})

    # 1. 检查活动时间
    activity = Activity.query.first()
    if activity:
        now = datetime.now()
        if activity.start_time and now < activity.start_time:
            return jsonify({'msg': '活动尚未开始', 'code': 400})
        if activity.end_time and now > activity.end_time:
            return jsonify({'msg': '活动已经结束', 'code': 400})

    # 2. 检查题目是否存在
    riddle = Riddle.query.get(riddle_id)
    if not riddle:
        return jsonify({'msg': '题目不存在', 'code': 404})

    # 3. 检查是否已被抢答
    if riddle.is_solved:
        return jsonify({
            'success': False,
            'msg': f'太可惜了，这道题已经被 {riddle.solver.username if riddle.solver else "别人"} 抢先猜中了！',
            'code': 400
        })

    # 判断我是不是已经回答过
    if GuessRecord.query.filter_by(user_id=g.current_user.id, riddle_id=riddle_id).first():
        return jsonify({'success': False, 'msg': '你已经猜过该题了！' , 'code': 400})

    # 4. 比对答案 (忽略大小写)
    if user_answer.lower() == riddle.answer.lower():
        # --- 再次检查防止并发 ---
        if riddle.is_solved:
            return jsonify({'success': False, 'msg': '手慢了，已被抢答！' , 'code': 400})

        # 标记为已解决
        riddle.is_solved = True
        riddle.solver_id = g.current_user.id  # g.current_user 来自 login_required

        # 写入记录
        record = GuessRecord(user_id=g.current_user.id, riddle_id=riddle_id,is_solved=True)
        db.session.add(record)

        try:
            db.session.commit()
            return jsonify({'success': True, 'msg': '恭喜你！抢答成功！', 'code': 200})
        except Exception as e:
            db.session.rollback()
            return jsonify({'success': False, 'msg': '系统繁忙，请重试'}), 500
    else:
        # 写入记录
        record = GuessRecord(user_id=g.current_user.id, riddle_id=riddle_id,is_solved=False)
        db.session.add(record)
        db.session.commit()
        return jsonify({'success': False, 'msg': '答案不对，请再接再厉！'})


# ============================
# 7. 查看我猜中的灯谜记录
# ============================
@client_bp.route('/my/records', methods=['GET'])
@login_required
def my_records():
    # 查询当前用户的所有战绩
    records = GuessRecord.query.filter_by(user_id=g.current_user.id).order_by(GuessRecord.solve_time.desc()).all()
    return jsonify([r.to_dict() for r in records])


