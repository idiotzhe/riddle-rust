
<template>
  <div class="app-container">
    <TopHeader />
    
    <main class="main-card">
      <div class="nav-container">
        <nav class="tab-nav">
          <router-link to="/riddles" class="tab-item">
            <span class="icon"><i class="iconfont icon-deng"></i></span> 灯谜管理
          </router-link>
          <router-link to="/activity" class="tab-item">
            <span class="icon"><i class="iconfont icon-deng"></i></span> 活动管理
          </router-link>
          <router-link to="/records" class="tab-item">
            <span class="icon"><i class="iconfont icon-deng"></i></span> 答题记录
          </router-link>
          <router-link to="/users" class="tab-item">
            <span class="icon"><i class="iconfont icon-deng"></i></span> 用户管理
          </router-link>
        </nav>
      </div>

      <div class="content-view" :class="{ 'content-activity': isActivity }">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>
</template>

<script setup>
import TopHeader from './TopHeader.vue';

import { useRoute } from 'vue-router'
import { computed } from 'vue'

const route = useRoute()

const isActivity = computed(() => route.path === '/activity')
</script>

<style scoped>
</style>