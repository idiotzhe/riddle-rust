import { createRouter, createWebHashHistory } from 'vue-router';
import RiddleManage from '../views/RiddleManage.vue';
import ActivityManage from '../views/ActivityManage.vue';
import RecordManage from '../views/RecordManage.vue';
import UserManage from '../views/UserManage.vue';

const routes = [
  {
    path: '/',
    redirect: '/riddles'
  },
  {
    path: '/riddles',
    name: 'RiddleManage',
    component: RiddleManage,
    meta: { title: '灯谜管理', icon: 'riddle' }
  },
  {
    path: '/activity',
    name: 'ActivityManage',
    component: ActivityManage,
    meta: { title: '活动管理', icon: 'activity' }
  },
  {
    path: '/records',
    name: 'RecordManage',
    component: RecordManage,
    meta: { title: '答题记录', icon: 'record' }
  },
  {
    path: '/users',
    name: 'UserManage',
    component: UserManage,
    meta: { title: '用户管理', icon: 'user' }
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;