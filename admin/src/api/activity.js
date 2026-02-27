import request from './request'

/**
 * 获取活动配置
 */
export function getActivityConfig() {
  return request({
    url: '/activity',
    method: 'get'
  })
}

/**
 * 修改活动配置
 * @param {Object} data { name, start_time, end_time }
 */
export function updateActivityConfig(data) {
  return request({
    url: '/activity',
    method: 'post',
    data
  })
}