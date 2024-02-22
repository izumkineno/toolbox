import { defineStore } from 'pinia'


interface ICom {
    label: string
    value: string
    disabled?: boolean
}


export const base_control = defineStore('base_control', () => {
    const serialPorts: ICom[] = []
    const bauds_rate = [9600, 19200, 38400, 57600, 115200]
    const serialBauds: ICom[] = bauds_rate.map(v => {
        return {
            label: v.toString(),
            value: v.toString()
        }
    })

    return {
        serialPorts,
        serialBauds
    }
})