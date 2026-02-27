<script setup>
import { ref, onMounted } from 'vue';
import { Search, Setting, Download } from '@element-plus/icons-vue';
import { getRecordList, exportRecords } from '../api/record';
import { ElMessage } from 'element-plus';

const records = ref([]);
const loading = ref(false);
const total = ref(0);
const queryParams = ref({
  page: 1,
  pageSize: 10,
  keyword: ''
});

const fetchList = async () => {
  loading.value = true;
  try {
    const data = await getRecordList(queryParams.value);
    records.value = data.list || [];
    total.value = data.total || 0;
  } catch (error) {
    console.error('Failed to fetch records:', error);
  } finally {
    loading.value = false;
  }
};

const handleExport = async () => {
  try {
    const blob = await exportRecords(queryParams.value);
    const url = window.URL.createObjectURL(new Blob([blob]));
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', `records-${new Date().getTime()}.xlsx`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    ElMessage.success('导出成功');
  } catch (error) {
    console.error('Export failed:', error);
  }
};

onMounted(() => {
  fetchList();
});
</script>

<template>
  <div class="view-container view-container-record view-container-offset">
    <div class="toolbar">
      <div class="search-wrapper">
        <el-input
          v-model="queryParams.keyword"
          placeholder="搜索姓名"
          class="gf-search-input"
          :prefix-icon="Search"
          @keyup.enter="fetchList"
          clearable
          @clear="fetchList"
        />
      </div>
      <div class="action-buttons">
        <el-button class="gf-btn-filter" :icon="Setting" @click="fetchList">刷新</el-button>
        <el-button type="primary" class="gf-btn-export" :icon="Download" @click="handleExport">导出</el-button>
      </div>
    </div>

    <el-table 
      v-loading="loading"
      :data="records" 
      class="gf-el-table" 
      style="width: 100%" 
      :header-cell-style="{ background: '#FFDDCB', color: '#5D4037', fontWeight: '900', fontSize: '1.2rem', padding: '20px 0', textAlign: 'center' }"
      :cell-style="{ textAlign: 'center', fontSize: '1.1rem', color: '#5D4037', padding: '15px 0' }"
    >
      <el-table-column prop="id" label="记录ID" width="100" />
      <el-table-column prop="user_name" label="中奖用户" width="180" />
      <el-table-column prop="riddle_question" label="答对灯谜" min-width="250" show-overflow-tooltip />
      <el-table-column prop="solve_time" label="中奖时间" width="200" />
    </el-table>

    <div class="pagination-container">
      <el-pagination
        v-model:current-page="queryParams.page"
        v-model:page-size="queryParams.pageSize"
        background
        layout="prev, pager, next, jumper, ->, total"
        :total="total"
        class="gf-pagination"
        @current-change="fetchList"
      />
    </div>
  </div>
</template>

<style scoped>



:deep(.gf-search-input .el-input__wrapper) {
  border-radius: 50px;
  background-color: #FFFFFF;
  padding-left: 20px;
  height: 55px;
  box-shadow: inset 0 2px 6px rgba(0,0,0,0.05) !important;
  border: none;
}

:deep(.gf-search-input .el-input__icon) {
  color: #FF7043;
  font-size: 1.4rem;
}


:deep(.el-table) {
  background-color: transparent !important;
}

:deep(.el-table__row:hover > td) {
  background-color: #FFF3E0 !important;
}


:deep(.gf-pagination.is-background .el-pager li:not(.is-active)) {
  background-color: transparent;
}

:deep(.gf-pagination.is-background .el-pager li.is-active) {
  background-color: #D32F2F !important;
  color: white;
}
</style>