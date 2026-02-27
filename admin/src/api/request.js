import axios from 'axios'
import { ElMessage } from 'element-plus'

const service = axios.create({
  // 在 Tauri 中，我们需要指向 Axum 后端的完整地址
  baseURL: import.meta.env.VITE_API_BASE_URL || (import.meta.env.PROD ? 'http://localhost:9000/pro-api' : '/pro-api'),
  timeout: 5000
})

// Request interceptor
service.interceptors.request.use(
  config => {
    // You can add token to headers here
    // const token = localStorage.getItem('token')
    // if (token) {
    //   config.headers['Authorization'] = `Bearer ${token}`
    // }
    return config
  },
  error => {
    console.log(error)
    return Promise.reject(error)
  }
)

// Response interceptor
service.interceptors.response.use(
  response => {
    const res = response.data
    // If your backend returns a custom code, handle it here
    // If 'code' is missing, we assume it's a flat object or list and return it directly
    if (res.code !== undefined && res.code !== 200 && res.code !== 0 && !response.config.rawResponse) {
      ElMessage({
        message: res.message || 'Error',
        type: 'error',
        duration: 5 * 1000
      })
      return Promise.reject(new Error(res.message || 'Error'))
    } else {
      // If res.code is 200/0, return res.data. If res.code is missing, return res.
      return res.data !== undefined ? res.data : res
    }
  },
  error => {
    console.log('err' + error)
    ElMessage({
      message: error.message,
      type: 'error',
      duration: 5 * 1000
    })
    return Promise.reject(error)
  }
)

export default service
