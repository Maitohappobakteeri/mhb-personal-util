import { CommandConfig } from "../../config-builder/config-interface";

const config: CommandConfig = {
    name: "echo",
    description: 'Just to check this utility works!',
    commandToExecute: 'echo.bash'
}

export default config;