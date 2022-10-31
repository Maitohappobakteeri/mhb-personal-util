import { CommandConfig } from "../../config-builder/config-interface";

const config: CommandConfig = {
    name: "task-switch-alarm",
    description: 'TUI App that notifies that you should do something else and stretch now!',
    commandToExecute: 'task_switch_alarm.bash'
}

export default config;