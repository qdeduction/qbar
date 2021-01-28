import { commands, workspace, ExtensionContext } from 'vscode'
import { LanguageClient } from 'vscode-languageclient/node'

let client: LanguageClient

function startClient (): void {
  const path: string = workspace.getConfiguration('qbar').get('executablePath') ?? 'qbar'
  client = new LanguageClient('qbar', 'qbar language server',
    {
      run: { command: path, args: ['server'] },
      debug: { command: path, args: ['server', '--debug'] }
    },
    {
      documentSelector: [{ scheme: 'file', language: 'qbar' }],
      initializationOptions: { extraCapabilities: {} }
    })
  client.start()
}

export function activate (context: ExtensionContext): void {
  startClient()
  context.subscriptions.push(
    // TODO: workspace.onDidOpenTextDocument(),
    // TODO: workspace.onWillSaveTextDocument(),
    commands.registerCommand('qbar.shutdownServer',
      async () => await client.stop().then(() => {}, () => {})),
    commands.registerCommand('qbar.restartServer',
      async () => await client.stop().then(startClient, startClient))
  )
}

export function deactivate (): Thenable<void> | undefined {
  if (client === undefined) {
    return undefined
  }
  return client.stop()
}
