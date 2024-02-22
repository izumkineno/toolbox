
<script lang="ts" setup>
import {ref, watch} from 'vue'
import {Action, ElMessage, ElMessageBox, TabPaneName} from 'element-plus'
import Main_body from "./main_body.vue";

let tabIndex = 1
const tabCurName = ref('1')
const tabList = ref([
  {
    title: '串口 1',
    name: '1',
    newMsg: false,
    com_name: "",
  },
])
// 去除新信息红点
watch([() => tabList.value, () => tabCurName.value], () => {
  tabList.value.forEach((tab) => {
    if (tab.name === tabCurName.value) {
      tab.newMsg = false
    }
  })
}, {
  deep: true
})


const tab_close_alert = (targetName: TabPaneName | undefined, ) => {
  const name = tabList.value.filter(value => {
    return value.name === targetName
  })[0].title
  ElMessageBox.alert(
      `确定要关闭标签 <strong> ${name} </strong> ？`,
      '',
      {
        dangerouslyUseHTMLString: true,
        confirmButtonText: '确定',
        callback: (action: Action) => {
          if (action == 'confirm') {
            const tabs = tabList.value
            let activeName = tabCurName.value
            if (activeName === targetName) {
              tabs.forEach((tab, index) => {
                if (tab.name === targetName) {
                  const nextTab = tabs[index + 1] || tabs[index - 1]
                  if (nextTab) {
                    activeName = nextTab.name
                  }
                }
              })
            }

            tabCurName.value = activeName
            tabList.value = tabs.filter((tab) => tab.name !== targetName)
          }
        },
  })
}
const handleTabsEdit = (targetName: TabPaneName | undefined, action: 'remove' | 'add') => {
  if (action === 'add') {
    const newTabName = `${++tabIndex}`
    tabList.value.push({
      title: '串口 ' + newTabName,
      name: newTabName,
      newMsg: false,
      com_name: "",
    })
    tabCurName.value = newTabName
  } else if (action === 'remove') {
    tab_close_alert(targetName)
  }
}


</script>

<template>
  <el-tabs
      v-model="tabCurName"
      type="card"
      editable
      stretch
      @edit="handleTabsEdit">
    <el-tab-pane v-for="item in tabList" lazy :key="item.name" :name="item.name">
      <template #label>
        <el-badge is-dot style="padding: 3px" :hidden="!item.newMsg">{{ item.title + ` (${item.com_name})` }}</el-badge>
      </template>
      <main_body :key="item.name" v-model:new-msg="item.newMsg" v-model:com-name="item.com_name" />
    </el-tab-pane>
  </el-tabs>
</template>

<style>



</style>
