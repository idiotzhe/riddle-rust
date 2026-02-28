<script setup>
import { ref, onMounted, reactive, nextTick } from 'vue';
import { Search, Edit, Delete } from '@element-plus/icons-vue';
import { getUserList, deleteUser } from '../api/user';
import { ElMessage, ElMessageBox } from 'element-plus';

const tableRef = ref(null);
const users = ref([]);
const loading = ref(false);
const total = ref(0);
const queryParams = ref({
  page: 1,
  pageSize: 10,
  keyword: ''
});
const domain = ref(window.location.origin);


const showModal = ref(false);
const submitting = ref(false);
const currentForm = reactive({
  id: null,
  username: '',
  avatar: ''
});

const fetchList = async () => {
  loading.value = true;
  try {
    const data = await getUserList(queryParams.value);
    users.value = data.list || [];
    total.value = data.total || 0;
    // 重置滚动条
    nextTick(() => {
      if (tableRef.value) {
        tableRef.value.setScrollTop(0);
      }
    });
  } catch (error) {
    console.error('Failed to fetch users:', error);
    ElMessage.error('获取用户列表失败');
  } finally {
    loading.value = false;
  }
};

const handleEdit = (row) => {
  currentForm.id = row.id;
  currentForm.username = row.username;
  currentForm.avatar = row.avatar;
  showModal.value = true;
};

const handleSubmit = async () => {
    showModal.value = false;
  // if (!currentForm.username) {
  //   ElMessage.warning('请输入用户名');
  //   return;
  // }
  //
  // submitting.value = true;
  // try {
  //   await updateUser(currentForm);
  //   ElMessage.success('更新成功');
  //   showModal.value = false;
  //   fetchList();
  // } catch (error) {
  //   console.error('Update failed:', error);
  // } finally {
  //   submitting.value = false;
  // }
};

const handleDelete = (row) => {
  ElMessageBox.confirm('确认删除该用户吗？', '提示', {
    type: 'warning'
  }).then(async () => {
    try {
      await deleteUser(row.id);
      ElMessage.success('删除成功');
      fetchList();
    } catch (error) {
      console.error('Delete failed:', error);
    }
  });
};

onMounted(() => {
  fetchList();
});
</script>

<template>
  <div class="view-container view-container-offset user-view-container">
    <div class="toolbar">
      <div class="search-wrapper">
        <el-input
          v-model="queryParams.keyword"
          placeholder="搜索用户名"
          class="gf-search-input"
          :prefix-icon="Search"
          @keyup.enter="fetchList"
          clearable
          @clear="fetchList"
        />
      </div>
    </div>

    <!-- Edit User Modal -->
    <el-dialog
      v-model="showModal"
      title="编辑用户信息"
      width="500px"
      class="gf-dialog"
      destroy-on-close
      center
    >
      <el-form label-position="top" class="gf-form">
        <el-form-item label="用户名" required>
          <el-input v-model="currentForm.username" placeholder="请输入用户名" class="gf-el-input" />
        </el-form-item>
        <el-form-item label="用户头像">
           <el-avatar :size="60" :src="`${domain}/${currentForm.avatar}`">
              {{ currentForm.username?.charAt(0) }}
           </el-avatar>
        </el-form-item>
      </el-form>
      <template #footer>
        <div class="modal-footer">
          <button class="gf-submit-btn" @click="handleSubmit" :disabled="submitting">
            {{ submitting ? '保存中...' : '关闭' }}
          </button>
        </div>
      </template>
    </el-dialog>

    <el-table 
      ref="tableRef"
      v-loading="loading"
      :data="users" 
      class="gf-el-table" 
      style="width: 100%" 
      :header-cell-style="{ background: '#FFDDCB', color: '#5D4037', fontWeight: '900', fontSize: '1.2rem', padding: '20px 0', textAlign: 'center' }"
      :cell-style="{ textAlign: 'center', fontSize: '1.1rem', color: '#5D4037', padding: '15px 0' }"
    >
      <el-table-column prop="id" label="ID" width="100" />
      <el-table-column prop="username" label="用户名" min-width="180" />
      <el-table-column label="用户头像" width="150">
        <template #default="scope">
          <el-avatar :size="50" :src="`${domain}/${scope.row.avatar}`">
            {{ scope.row.username?.charAt(0) }}
          </el-avatar>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="180">
        <template #default="scope">
          <div class="actions">
            <button class="circle-btn edit-btn" @click="handleEdit(scope.row)">
              <el-icon :size="20">
                <View />
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

<style scoped></style>