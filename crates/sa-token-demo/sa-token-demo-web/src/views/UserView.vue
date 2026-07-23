<script setup>
import { ref, onMounted } from 'vue'
import { listUsers } from '../api.js'

const users = ref([])
const error = ref('')

onMounted(async () => {
  try {
    const resp = await listUsers()
    users.value = resp.data || []
  } catch (err) {
    error.value = err.response?.data?.message || err.message
  }
})
</script>

<template>
  <div class="users">
    <h2>用户列表</h2>
    <p v-if="error" class="error">{{ error }}</p>
    <table v-else>
      <thead>
        <tr>
          <th>ID</th>
          <th>登录名</th>
          <th>角色</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="u in users" :key="u.id">
          <td>{{ u.id }}</td>
          <td>{{ u.name }}</td>
          <td>{{ (u.roles || []).join(', ') }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.users {
  margin: 1rem 0;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th, td {
  padding: 0.5rem;
  border: 1px solid #eee;
  text-align: left;
}
th {
  background: #f5f5f5;
}
.error {
  color: #f56c6c;
}
</style>
