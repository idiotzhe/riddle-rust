import request from './request'

/**
 * 获取用户列表
 * @param {Object} params { page, pageSize }
 */
export function getUserList(params) {
  return request({
    url: '/users',
    method: 'get',
    params
  })
}

/**
 * 删除用户
 * @param {number|string} id
 */
export function deleteUser(id) {
  return request({
    url: `/user/${id}`,
    method: 'delete'
  })
}

/**
 * 更新用户信息
 * @param {Object} data { id, username, avatar }
 */
export function updateUser(data) {
  return request({
    url: `/user/${data.id}`,
    method: 'put',
    data
  })
}
