import request from './request'

/**
 * 获取答题记录列表
 * @param {Object} params { page, pageSize, keyword }
 */
export function getRecordList(params) {
  return request({
    url: '/leaderboard',
    method: 'get',
    params
  })
}

/**
 * 获取抢答排行榜
 */
export function getLeaderboard() {
  return request({
    url: '/stats/leaderboard',
    method: 'get'
  })
}

/**
 * 导出记录报告
 * @param {Object} params
 */
export function exportRecords(params) {
  // 如果是本地保存模式，不要指定 blob，让 request.js 默认处理成 json
  const isSaveLocally = params && params.save_locally;
  return request({
    url: '/records/export',
    method: 'get',
    params,
    responseType: isSaveLocally ? 'json' : 'blob'
  })
}