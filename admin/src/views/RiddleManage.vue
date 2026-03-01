<script setup>
import { ref, onMounted, nextTick } from 'vue';
import RiddleModal from '../components/RiddleModal.vue';
import {Search, Plus, Edit, Delete, Upload} from '@element-plus/icons-vue';
import { getRiddleList, deleteRiddle,importRiddles } from '../api/riddle';
import { ElMessage, ElMessageBox } from 'element-plus';

const tableRef = ref(null);
const riddles = ref([]);
const loading = ref(false);
const total = ref(0);
const queryParams = ref({
  page: 1,
  pageSize: 10,
  keyword: ''
});

const showModal = ref(false);
const currentEditData = ref(null);

const fetchList = async () => {
  loading.value = true;
  try {
    const data = await getRiddleList(queryParams.value);
    riddles.value = data.list || [];
    total.value = data.total || 0;
    // 数据加载后重置滚动条到顶部
    nextTick(() => {
      if (tableRef.value) {
        tableRef.value.setScrollTop(0);
      }
    });
  } catch (error) {
    console.error('Failed to fetch riddles:', error);
  } finally {
    loading.value = false;
  }
};

const handleAdd = () => {
  currentEditData.value = null;
  showModal.value = true;
};

const handleEdit = (row) => {
  currentEditData.value = { ...row };
  showModal.value = true;
};

const handleDelete = (row) => {
  ElMessageBox.confirm('确认删除该灯谜吗？', '提示', {
    type: 'warning'
  }).then(async () => {
    try {
      await deleteRiddle(row.id);
      ElMessage.success('删除成功');
      fetchList();
    } catch (error) {
      console.error('Delete failed:', error);
    }
  });
};

const handleImport = () => {
  // 创建一个隐藏的 input
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.xlsx, .xls'; // 只允许 Excel

  input.onchange = async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    // 创建 FormData 对象
    const formData = new FormData();
    formData.append('file', file);

    loading.value = true;
    try {
      const res = await importRiddles(formData);
      // 根据你后端的返回结构判断
      console.log(res)
      if (res.code === 200) {
        ElMessage.success(res.msg || '导入成功');
        fetchList(); // 刷新列表
      } else {
        ElMessage.error(res.msg || '导入失败');
      }
    } catch (error) {
      console.error('Import failed:', error);
      ElMessage.error('导入过程中发生错误');
    } finally {
      loading.value = false;
      // 清空 input 确保下次选择同一文件还能触发 change
      input.value = '';
    }
  };
  input.click();
};


const handleAddSuccess = () => {
  // 如果是新增，回到第一页以查看最新添加的灯谜
  queryParams.value.page = 1;
  fetchList();
};

onMounted(() => {
  fetchList();
});
</script>
<template>
  <div class="view-container view-container-riddle view-container-offset ">
    <div class="toolbar">
      <div class="search-wrapper">
        <el-input
          v-model="queryParams.keyword"
          placeholder="搜索谜面"
          class="gf-search-input"
          :prefix-icon="Search"
          @keyup.enter="fetchList"
          clearable
          @clear="fetchList"
        />
      </div>
      <el-button type="primary" class="gf-btn-add" @click="handleAdd" :icon="Plus">
        添加灯谜
      </el-button>
       <el-button type="primary" class="gf-btn-export" :icon="Upload" @click="handleImport">导入</el-button>
    </div>

    <RiddleModal v-model="showModal" :edit-data="currentEditData" @success="handleAddSuccess" />

    <el-table 
      ref="tableRef"
      v-loading="loading"
      :data="riddles" 
      class="gf-el-table" 
      style="width: 100%" 
      :header-cell-style="{ background: '#FFDDCB', color: '#5D4037', fontWeight: '900', fontSize: '1.2rem', padding: '20px 0', textAlign: 'center' }"
      :cell-style="{ textAlign: 'center', fontSize: '1.1rem', color: '#5D4037', padding: '15px 0' }"
    >
      <el-table-column prop="question" label="谜面" min-width="220" show-overflow-tooltip />
      <el-table-column prop="answer" label="谜底" width="160" />
      <el-table-column label="选项" min-width="200">
        <template #default="scope">
          <div class="options-tags">
            <span v-for="(opt, index) in scope.row.options" :key="index" class="custom-tag orange-tag">
              {{ opt }}
            </span>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="120">
        <template #default="scope">
          <span :class="['custom-tag', scope.row.is_solved ? 'green-tag' : 'gray-tag']">
            {{ scope.row.is_solved ? '已解决' : '未解决' }}
          </span>
        </template>
      </el-table-column>
      <el-table-column prop="add_time" label="创建时间" width="180" />
      <el-table-column label="操作" width="160">
        <template #default="scope">
          <div class="actions">
            <button class="circle-btn edit-btn" @click="handleEdit(scope.row)">
              <el-icon :size="20">
                <Edit />
              </el-icon>
            </button>
            <button class="circle-btn delete-btn" @click="handleDelete(scope.row)">
              <el-icon :size="20">
                <Delete />
              </el-icon>
            </button>
          </div>
        </template>
      </el-table-column>
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

:deep(.gf-pagination.is-background .el-pager li.is-active) {
  background-color: #D32F2F !important;
  color: white;
}
</style>