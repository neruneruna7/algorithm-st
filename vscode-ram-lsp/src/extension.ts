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

export function activate(context: vscode.ExtensionContext): void {
  const serverPath = resolveServerPath(context);

  const serverOptions: ServerOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "ram" }],
  };

  client = new LanguageClient("ramLsp", "RAM LSP", serverOptions, clientOptions);
  context.subscriptions.push(client);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}

function resolveServerPath(context: vscode.ExtensionContext): string {
  const configuredPath = vscode.workspace
    .getConfiguration("ramLsp")
    .get<string>("serverPath");

  if (configuredPath && configuredPath.length > 0) {
    return configuredPath;
  }

  const binaryName = process.platform === "win32" ? "ram-lsp.exe" : "ram-lsp";
  const workspaceBinaryPath = path.resolve(
    context.extensionPath,
    "..",
    "target",
    "debug",
    binaryName
  );

  if (fs.existsSync(workspaceBinaryPath)) {
    return workspaceBinaryPath;
  }

  return binaryName;
}
