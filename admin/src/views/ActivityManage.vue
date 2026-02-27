<script setup>
import { ref, reactive, onMounted } from 'vue';
import { getActivityConfig, updateActivityConfig } from '../api/activity';
import { ElMessage } from 'element-plus';

const loading = ref(false);
const form = reactive({
  name: '',
  start_time: '',
  end_time: ''
});

const timeRange = ref([]);

const fetchConfig = async () => {
  try {
    const data = await getActivityConfig();
    form.name = data.name;
    form.start_time = data.start_time;
    form.end_time = data.end_time;
    if (form.start_time && form.end_time) {
      timeRange.value = [form.start_time, form.end_time];
    }
  } catch (error) {
    console.error('Failed to fetch activity config:', error);
  }
};

const handleSave = async () => {
  if (timeRange.value && timeRange.value.length === 2) {
    form.start_time = timeRange.value[0];
    form.end_time = timeRange.value[1];
  } else {
    ElMessage.warning('请选择起止时间');
    return;
  }

  loading.value = true;
  try {
    await updateActivityConfig(form);
    ElMessage.success('保存成功');
  //   新窗口打开
  } catch (error) {
    console.error('Save failed:', error);
  } finally {
    loading.value = false;
  }
};

// const openActivity = () => {
  // window.location.href =`${window.location.origin}/frontend/index`;
  // window.location.href =`${window.location.origin}/frontend/index`;
// };
const openActivity = () => {
  // 直接在当前窗口跳转到活动大屏页面
   window.location.href = `${window.location.origin}/frontend/index`;
};



onMounted(() => {
  fetchConfig();
});
</script>

<template>
  <div class="view-container activity-view-container">
    <div class="status-bar-el">
      <span class="status-label">活动设置</span>
      <span class="status-tip">配置活动名称与生效时间</span>
    </div>

    <div class="content-split">
      <div class="config-panel" v-loading="loading">
        <el-form label-position="top">
          <el-form-item label="活动名称" required>
            <el-input v-model="form.name" placeholder="请输入活动名称..." class="gf-el-input" />
          </el-form-item>

          <el-form-item label="起止时间" required>
            <el-date-picker
              v-model="timeRange"
              type="datetimerange"
              range-separator="至"
              start-placeholder="开始时间"
              end-placeholder="结束时间"
              value-format="YYYY-MM-DD HH:mm:ss"
              class="gf-el-date-picker"
            />
          </el-form-item>
          
          <div class="tc"><el-button type="primary" class="gf-submit-btn save-btn-el" @click="handleSave">保存配置</el-button></div>
          <div class="tc"><el-button type="primary" class="gf-submit-btn save-btn-el" @click="openActivity">开启活动</el-button></div>
        </el-form>
      </div>

      
    </div>
  </div>
</template>

<style scoped>

:deep(.el-form-item__label) {
  color: var(--theme-red-light) !important;
  font-weight: bold !important;
  font-size: 18px !important;
}

:deep(.el-input__wrapper), :deep(.el-textarea__inner) {
  box-shadow: none !important;
  border-radius: 50px !important;
  height: 50px;
  color: var(--theme-red);
  font-size: 16px;
  padding-left: 20px;
  padding-right: 20px;
}

</style>