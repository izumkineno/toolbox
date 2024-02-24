
<script setup lang="ts">
  import {invoke_toast} from "../util";
  import {base_control} from "../store/base_control";
  import {onBeforeUnmount, reactive, ref, watch} from "vue";
  import {listen} from "@tauri-apps/api/event";
  import {Delete} from "@element-plus/icons-vue";
  import {appWindow} from "@tauri-apps/api/window";

  const count = reactive({
    send: 0,
    recv: 0,
  })
  const char_codes = [
    {
      label: "UTF8",
      value: 0
    },
    {
      label: "GBK",
      value: 1
    }
  ]
  const recv_length = [
    {
      label: "无限制",
      value: "-1"
    },
    {
      label: "不显示",
      value: "0"
    },
    {
      label: "10",
      value: "10"
    }
  ]

  /// 串口配置相关

  const sp = base_control();
  const info_sp = reactive({
    id: Math.random().toString(36).slice(2),
    serialPort: '',
    baudRate: '',
  })
  const refresh_ports = () => {
    if (!info_connect.state) {
      invoke_toast("get_serial_ports").then(v => {
        console.debug(v)
        sp.serialPorts =(v as {name: string, state: boolean}[]).map(v => {
          return  {
            value: v.name,
            label: v.name,
            disabled: !v.state
          }
        })
      })
    }
  }



  // todo:数据量过大时会导致内存占用过高，考虑使用虚拟列表
  /// 串口连接和数据接收相关

  const recv_window = ref()
  const recv_window_outer = ref()
  const info_connect = reactive({
    // 连接状态
    state: false,
    // 连接加载
    loading: false,
    // 缓冲区
    buffer: "",
    // 是否是十六进制
    hex: false,
    // 是否显示时间
    show_time: false,
    // 编码,
    char_code: 0,
    // 显示消息的最大长度
    recv_len: "-1",
    // 接收监听句柄
    listen_handle: () => {},
  })
  const connect = async () => {
    info_connect.loading = true
    return invoke_toast("connect", {
      id: info_sp.id,
      port: info_sp.serialPort,
      br: parseInt(info_sp.baudRate)
    }).then(async () => {
      info_connect.state = true
      info_connect.listen_handle = await listen<{
        recv_count: number,
        send_count: number,
        msg: string,
      }>("recv_" + info_sp.id, (event) => {
        const data = event.payload

        info_connect.buffer = data.msg
        count.send = data.send_count
        count.recv = data.recv_count

        // 滑动到底部
        recv_window_outer.value.setScrollTop(recv_window.value.scrollHeight)
      })
    }).finally(() => {
      info_connect.loading = false
    })
  }
  const disconnect = () => invoke_toast("disconnect", {
    id: info_sp.id
  }).then(() => {
    info_connect.listen_handle()
    info_connect.state = false
  })
  const sw_connect = () => info_connect.state ? disconnect() : connect()



  // 设置接收信息

  const set_recv = (item: number, value: number) => {
    invoke_toast("set_recv_setting", {
      id: info_sp.id,
      item,
      value,
    })
  }

  const clear_buffer = () => {
    info_connect.buffer = ""
    set_recv(0, 0)
  }
  const set_hex = () => {
    info_connect.hex = !info_connect.hex
    set_recv(102, info_connect.hex ? 1 : 0)
  }
  const set_time = () => {
    info_connect.show_time = !info_connect.show_time
    set_recv(101, info_connect.show_time ? 1 : 0)
  }
  const set_char_code = () => {
    set_recv(103, info_connect.char_code)
  }
  const set_recv_len = () => {
    set_recv(104, parseInt(info_connect.recv_len))
  }
  const clear_recv_count = () => {
    count.recv = 0
    set_recv(0, 1)
  }
  const clear_send_count = () => {
    count.send = 0
    set_recv(0, 2)
  }


  /// 组件销毁后，清理后台事件

  const clear_link = () => {
    if (info_connect.state) {
      send_loop_clear()
      disconnect()
    }
  }
  onBeforeUnmount(() => {
    clear_link()
  })
  window.addEventListener('beforeunload', () => {
    clear_link()
  });



  /// 新消息红点和标题串口
  const com_name = defineModel<string>("comName")
  watch([() => info_sp.serialPort, () => info_connect.state], ([n1, n2], _) => {
    com_name.value = n2 ? n1 : ""
  })
  const newMsg = defineModel<boolean>("newMsg")
  watch(() => info_connect.buffer, () => {
    newMsg.value = true
  })



  /// 发送数据相关

  const info_send = reactive({
    buffer: "",
    hex: false,
    loop: false,
    loop_time: "100",
    end: '',
    end_option: [
      {
        label: "无",
        value: ""
      },
      {
        label: "\\n",
        value: "\n"
      },
      {
        label: "\\r",
        value: "\r"
      },
      {
        label: "\\r\\n",
        value: "\r\n"
      },
      {
        label: "\\n\\r",
        value: "\n\r"
      }
    ]
  })
  const send = () => {
    if (info_connect.state) {
      const msg = {
        type: info_send.hex ? "hex" : "str",
        loop: info_send.loop,
        loop_time: parseInt(info_send.loop_time),
        msg: info_send.hex
            ? info_send.buffer.trim().split(" ")
                .map(v => {
                  return parseInt(v, 16)
                })
                .filter(v => {
                  return !isNaN(v) && v >= 0 && v <= 255
                })
            : info_send.buffer + info_send.end
      }
      console.log(msg)
      appWindow.emit(`send_${info_sp.id}`, msg)
      count.send += msg.msg.length
    }
    else {
      console.log("未连接")
    }
  }
  const clear_send = () => {
    info_send.buffer = ""
  }

  const send_loop_clear = () => appWindow.emit(`send_loop_${info_sp.id}`, "close")
  const send_loop = (state: boolean) => {
    if (state) {
      send_loop_clear()
    }
    send()
  }
  watch(() => info_send.loop, (_, o) => {
    console.log(o, info_connect.state)
    send_loop(o)
  })

</script>

<template>
  <div class="base_control space_between">
    <div class="base_control_left">
      <el-select
          @click="refresh_ports"
          v-model="info_sp.serialPort"
          placeholder="串口"
          :disabled="info_connect.state"
          size="small"
          style="width: 100px">
        <el-option
            v-for="item in sp.serialPorts"
            :key="item.value"
            :label="item.label"
            :value="item.value"
            :disabled="item.disabled"
        />
      </el-select>
      <el-select
          :disabled="info_connect.state"
          v-model="info_sp.baudRate"
          placeholder="波特率"
          size="small"
          style="width: 100px"
          filterable
          allow-create>
        <el-option
            v-for="item in sp.serialBauds"
            :key="item.value"
            :label="item.label"
            :value="item.value"
            :disabled="item.disabled"
        />
      </el-select>
      <el-switch
          v-model="info_connect.state"
          :loading="info_connect.loading"
          :before-change="sw_connect"
          style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949"
      />
    </div>
    <div>

    </div>
  </div>
  <div class="data_recv_send">
    <div class="space_between">
      <el-space>
        <el-text type="info" size="large" style="font-weight: bolder">接收区</el-text>
        <el-tag type="success" class="count" @click="clear_recv_count">{{ count.recv }}</el-tag>
      </el-space>
      <el-space>
        <el-checkbox label="Hex" @click="set_hex" />
        <el-checkbox label="时间" @click="set_time" />
        <el-select v-model="info_connect.char_code" style="width: 6rem" @change="set_char_code">
          <el-option v-for="item of char_codes" :key="item.value" :label="item.label" :value="item.value" />
        </el-select>
        <el-select v-model="info_connect.recv_len" style="width: 6rem" @change="set_recv_len" filterable allow-create>
          <el-option v-for="item of recv_length" :key="item.value" :label="item.label" :value="item.value" />
        </el-select>
        <el-button @click="clear_buffer" size="small" :icon="Delete" circle />
      </el-space>
    </div>
    <el-scrollbar class="recv-window-view" ref="recv_window_outer">
      <div class="recv-window" ref="recv_window" v-html="info_connect.buffer" />
    </el-scrollbar>
  </div>
  <div class="data_recv_send">
    <el-space >
      <el-text type="info" size="large" style="font-weight: bolder">发送区</el-text>
      <el-tag type="success" class="count" @click="clear_send_count">{{ count.send }}</el-tag>
    </el-space>
    <div class="data_send">
      <el-input v-model="info_send.buffer" type="textarea" rows="5" :placeholder="info_send.hex ? '输入十六进制数用空格隔开 例如： 1 02 aa b 1c' : ''" />
      <div class="data_send">
        <div class="send-control-l">
          <el-button type="primary" @click="send" >发送</el-button>
          <el-button type="info" @click="clear_send" >清空</el-button>
          <el-checkbox label="Hex" v-model="info_send.hex" />
        </div>
        <div class="send-control-r">
          <div style="display: flex; flex-direction: column; gap: 0.1rem">
            <el-checkbox label="周期发送(ms)" v-model="info_send.loop"  />
            <el-input type="number" step="1" v-model="info_send.loop_time" />
            <el-select placeholder="后缀" :disabled="info_send.hex" v-model="info_send.end" default-first-option>
              <el-option v-for="item of info_send.end_option" :key="item.value" :label="item.label" :value="item.value" />
            </el-select>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
  .space_between {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .base_control_left {
    display: flex;
    align-items: center;
    gap: var(--gap-size);
  }
  .data_recv_send {
    display: flex;
    flex-direction: column;
    gap: var(--gap-size);
  }
  .data_send {
    display: flex;
    gap: 0.5rem;
  }
  .send-control-l, .send-control-r {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    justify-content: center;
    align-items: baseline
  }
  .count {
    cursor: pointer;
  }
  .recv-window-view {
    height: 17.5rem;
    overflow: auto;
    border: var(--el-input-border-color, var(--el-border-color)) 1px solid;
    transition: border-color var(--el-transition-duration-fast);
  }
  .recv-window-view:hover {
    border-color: var(--el-color-primary);
  }
  .recv-window {
    width: 100%;
    word-break: break-word;
    white-space: pre-wrap;
    font-weight: normal;
    box-sizing: border-box;
    padding: 0.5rem 0.5rem 1.5rem;
    line-height: 1.5;
  }
</style>