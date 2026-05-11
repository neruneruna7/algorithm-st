import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel | undefined;

export function activate(context: vscode.ExtensionContext): void {
  outputChannel = vscode.window.createOutputChannel("RAM LSP");
  context.subscriptions.push(outputChannel);

  const serverPath = resolveServerPath(context);
  outputChannel.appendLine(`Starting RAM LSP from: ${serverPath}`);

  const serverOptions: ServerOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "ram" }],
  };

  client = new LanguageClient("ramLsp", "RAM LSP", serverOptions, clientOptions);
  context.subscriptions.push(client);
  client.start().catch((error) => {
    const message =
      error instanceof Error ? error.message : `Unknown error: ${String(error)}`;
    outputChannel?.appendLine(`Failed to start RAM LSP: ${message}`);
    void vscode.window.showErrorMessage(`Failed to start RAM LSP: ${message}`);
  });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}

function resolveServerPath(context: vscode.ExtensionContext): string {
  const configuredPath = vscode.workspace
    .getConfiguration("ramLsp")
    .get<string>("serverPath");

  // 明示設定がある場合は、それを最優先する。
  // これにより、開発中の debug build や任意の ram-lsp を手元で差し替えられる。
  if (configuredPath && configuredPath.length > 0) {
    return configuredPath;
  }

  const binaryName = process.platform === "win32" ? "ram-lsp.exe" : "ram-lsp";
  const bundledBinaryPath = path.join(
    context.extensionPath,
    "server",
    `${process.platform}-${process.arch}`,
    binaryName
  );

  // .vsix に同梱した language server があれば、それを使う。
  // 通常利用ではこのパスが使われるため、利用者は Rust toolchain を用意しなくてよい。
  if (fs.existsSync(bundledBinaryPath)) {
    ensureExecutable(bundledBinaryPath);
    return bundledBinaryPath;
  }

  const workspaceBinaryPath = path.resolve(context.extensionPath, "..", "target", "debug", binaryName);

  // Extension Development Host では、リポジトリ直下の target/debug/ram-lsp を
  // そのまま使えるようにしておく。
  if (fs.existsSync(workspaceBinaryPath)) {
    return workspaceBinaryPath;
  }

  // 最後の fallback として PATH 検索に任せる。
  // 失敗時は vscode-languageclient 側から起動エラーとして表示される。
  return binaryName;
}

function ensureExecutable(filePath: string): void {
  if (process.platform === "win32") {
    return;
  }

  try {
    fs.chmodSync(filePath, 0o755);
  } catch (error) {
    const message =
      error instanceof Error ? error.message : `Unknown error: ${String(error)}`;
    outputChannel?.appendLine(`Failed to chmod bundled RAM LSP: ${message}`);
  }
}
