import request from './request'

/**
 * 获取灯谜列表
 * @param {Object} params { page, pageSize }
 */
export function getRiddleList(params) {
  return request({
    url: '/riddles',
    method: 'get',
    params
  })
}

/**
 * 新增或修改灯谜
 * @param {Object} data { id, question, answer, options, remark, reset_status }
 */
export function saveRiddle(data) {
  return request({
    url: '/riddles',
    method: 'post',
    data
  })
}


/**
 * 删除灯谜
 * @param {number|string} id
 */
export function deleteRiddle(id) {
  return request({
    url: `/riddle/${id}`,
    method: 'delete'
  })
}

// src/api/riddle.js

export function importRiddles(formData) {
  return request({
    url: '/riddles/import', // 对应你 Flask 写的路由
    method: 'post',
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data'
    }
  });
}