const fs = require("fs");
const path = require("path");

const binaryName = process.platform === "win32" ? "ram-lsp.exe" : "ram-lsp";
const repositoryRoot = path.resolve(__dirname, "..", "..");
const sourcePath = path.join(repositoryRoot, "target", "release", binaryName);
const targetDirectory = path.join(
  __dirname,
  "..",
  "server",
  `${process.platform}-${process.arch}`
);
const targetPath = path.join(targetDirectory, binaryName);

if (!fs.existsSync(sourcePath)) {
  throw new Error(
    `Missing ${sourcePath}. Run \`cargo build -p ram-lsp --release\` from the repository root first.`
  );
}

fs.mkdirSync(targetDirectory, { recursive: true });
fs.copyFileSync(sourcePath, targetPath);

if (process.platform !== "win32") {
  fs.chmodSync(targetPath, 0o755);
}

console.log(`Bundled ${sourcePath} -> ${targetPath}`);
