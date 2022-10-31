import { CommandConfig } from "../config-builder/config-interface";
import echo from "./echo/mhb-config";
import taskSwitchAlarm from "./task_switch_alarm/mhb-config";

export const CONFIG_LIST: CommandConfig[] = [
    echo,
    taskSwitchAlarm
]