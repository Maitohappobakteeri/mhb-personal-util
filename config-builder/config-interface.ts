export interface CommandConfig {
    name: string
    description: string,
    commandToExecute: string
    subCommands?: CommandConfig[]
}