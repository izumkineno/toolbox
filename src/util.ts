import { invoke, InvokeArgs } from '@tauri-apps/api/tauri';
import {ElNotification} from "element-plus";

const offset = 20

export const invoke_toast = (cmd: string, args?: InvokeArgs | undefined, msg?: string) => {
    return new Promise((resolve, reject) => {
        let info = ElNotification({
            title: `调用指令 ${cmd}`,
            customClass: 'notification',
            offset: offset,
            duration: 0,
            type: 'info'
        })
        invoke(cmd, args).then(v => {
            ElNotification({
                title: `调用指令 ${cmd} 成功`,
                customClass: 'notification',
                offset: offset,
                duration: 3000,
                type: 'success'
            })
            resolve(v)
        }).catch(e => {
            ElNotification({
                title: `调用指令 ${cmd} 失败`,
                customClass: 'notification',
                message: `${e}`,
                offset: offset,
                duration: 3000,
                type: 'error'
            })
            reject(e)
        }).finally(() => {
            info.close()
        })
    })
}
