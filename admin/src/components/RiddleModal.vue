<template>
  <el-dialog
    v-model="visible"
    :title="form.id ? '修改灯谜' : '添加新灯谜'"
    width="550px"
    class="gf-dialog"
    :show-close="true"
    destroy-on-close
    center
  >
    <el-form label-position="top" class="gf-form" v-loading="loading">
      <el-form-item label="谜面" required>
        <el-input
          type="textarea"
          v-model="form.question"
          placeholder="请输入谜面..."
          :rows="4"
          class="gf-el-input"
        />
      </el-form-item>

      <el-form-item label="提示" required>
        <el-input v-model="form.remark" placeholder="请输入提示..." class="gf-el-input" />
      </el-form-item>
      
      <el-form-item label="谜底" required>
        <el-input v-model="form.answer" placeholder="请输入谜底..." class="gf-el-input" />
      </el-form-item>

      <el-form-item label="选项 (逗号或空格分隔)">
        <el-input v-model="optionsStr" placeholder="例如: 选项A, 选项B, 选项C" class="gf-el-input" />
      </el-form-item>

      <div class="form-row" v-if="form.id">
        <el-form-item label="状态重置">
          <el-checkbox v-model="form.reset_status">重置为未解决状态</el-checkbox>
        </el-form-item>
      </div>
    </el-form>

    <template #footer>
      <div class="modal-footer">
        <button class="gf-submit-btn" @click="handleSubmit" :disabled="loading">
          {{ loading ? '保存中...' : '保存' }}
        </button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup>
import { computed, reactive, ref, watch } from 'vue';
import { saveRiddle } from '../api/riddle';
import { ElMessage } from 'element-plus';

const props = defineProps({
  modelValue: Boolean,
  editData: Object
});

const emit = defineEmits(['update:modelValue', 'success']);

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

const loading = ref(false);
const optionsStr = ref('');
const form = reactive({
  id: null,
  question: '',
  remark: '',
  answer: '',
  options: [],
  reset_status: false
});

const resetForm = () => {
  Object.assign(form, {
    id: null,
    question: '',
    remark: '',
    answer: '',
    options: [],
    reset_status: false
  });
  optionsStr.value = '';
};

watch(() => props.editData, (newVal) => {
  if (newVal) {
    form.id = newVal.id;
    form.question = newVal.question;
    form.answer = newVal.answer;
    form.remark = newVal.remark;
    form.options = newVal.options || [];
    optionsStr.value = form.options.join(', ');
    form.reset_status = false;
  } else {
    resetForm();
  }
}, { immediate: true });

const handleSubmit = async () => {
  if (!form.question || !form.answer) {
    ElMessage.warning('请填写谜面和谜底');
    return;
  }
  
  // Convert options string to array
  if (optionsStr.value) {
    form.options = optionsStr.value.split(/[,，\s]+/).filter(item => item.trim() !== '');
  } else {
    form.options = [];
  }

  loading.value = true;
  try {
    await saveRiddle(form);
    ElMessage.success(form.id ? '修改成功' : '添加成功');
    emit('success');
    visible.value = false;
    resetForm();
  } catch (error) {
    console.error('Submit failed:', error);
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped>
/* Dialog Container */
:deep(.gf-dialog) {
  background: #FFF9F2 !important;
  border-radius: 40px !important;
  border: 4px solid #FFFFFF !important;
  border-top: 12px solid #FF5722 !important; 
  padding: 0 30px 20px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

:deep(.el-dialog__header) {
  margin-right: 0;
  padding-top: 35px;
  padding-bottom: 10px;
}

:deep(.el-dialog__title) {
  color: #8D4A2F !important;
  font-size: 2rem !important;
  font-family: "Kaiti", "STKaiti", serif !important;
  font-weight: bold !important;
  letter-spacing: 2px;
}

:deep(.el-dialog__headerbtn) {
  top: 35px;
  right: 35px;
}

:deep(.el-dialog__close) {
  font-size: 2rem;
  color: #8D4A2F !important;
  font-weight: bold;
}

/* Form Styles */
.gf-form {
  padding: 10px 10px;
}
:deep(.el-form-item__label) {
  color: #E64A19 !important;
  font-weight: bold !important;
  font-size: 18px !important;
  margin-bottom: 8px !important;
  line-height: 1.2;
}

:deep(.el-form-item.is-required:not(.is-no-asterisk)>.el-form-item__label:before) {
  color: #D32F2F !important;
  margin-right: 4px;
}

:deep(.el-input__wrapper), :deep(.el-textarea__inner) {
  background-color: #FBECE1;
  border-radius: 25px !important;
  
  box-shadow: inset 0 0 30px rgba(226,189,102,0.6);

  border: none !important;
  padding: 5px 20px !important;
  transition: all 0.3s;
}

:deep(.el-input__wrapper:hover), :deep(.el-textarea__inner:hover) {
  
}

:deep(.el-input__wrapper.is-focus), :deep(.el-textarea__inner:focus) {
  box-shadow: inset 0 0 30px rgba(226,189,102,0.6);
}

:deep(.el-textarea__inner) {
  border-radius: 20px !important;
  padding: 15px 25px !important;
  font-family: inherit;
}

:deep(.el-input__inner) {
  height: 50px;
  font-size: 16px;
}

/*:deep(.el-input__inner::placeholder), :deep(.el-textarea__inner::placeholder) {
  color: #A1887F;
}
*/
.modal-footer {
  text-align: center;
  padding-top: 10px;
  padding-bottom: 10px;
}

</style>