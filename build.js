import { spawnSync } from "child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function run(command, args, cwd) {
  console.log(`Running: ${command} ${args.join(" ")} in ${cwd || process.cwd()}`);
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: true,
    cwd,
  });
  if (result.status !== 0) {
    console.error(`Command failed with exit code ${result.status}`);
    process.exit(result.status);
  }
}

function copyRecursiveSync(src, dest) {
  const exists = fs.existsSync(src);
  const stats = exists && fs.statSync(src);
  const isDirectory = exists && stats.isDirectory();
  if (isDirectory) {
    if (!fs.existsSync(dest)) {
      fs.mkdirSync(dest, { recursive: true });
    }
    fs.readdirSync(src).forEach((childItemName) => {
      copyRecursiveSync(
        path.join(src, childItemName),
        path.join(dest, childItemName)
      );
    });
  } else {
    fs.copyFileSync(src, dest);
  }
}

const rootDir = __dirname;
const adminDir = path.join(rootDir, "admin");
const tauriDir = path.join(rootDir, "src-tauri");
const backendRustDir = path.join(rootDir, "backend-rust");

const isStandalone = process.argv.includes("--standalone");

if (isStandalone) {
  console.log("[1/3] Building Standalone Rust Backend...");
  run("cargo", ["build", "--release"], backendRustDir);

  console.log("[2/3] Preparing dist folder...");
  const distDir = path.join(rootDir, "dist");
  if (!fs.existsSync(distDir)) {
    fs.mkdirSync(distDir, { recursive: true });
  }

  const isWindows = process.platform === "win32";
  const binaryName = isWindows ? "backend-rust.exe" : "backend-rust";
  const targetBinaryName = isWindows ? "lantern-riddle-standalone.exe" : "lantern-riddle-standalone";

  const releaseDir = path.join(backendRustDir, "target", "release");
  const binaryPath = path.join(releaseDir, binaryName);

  if (fs.existsSync(binaryPath)) {
    fs.copyFileSync(binaryPath, path.join(distDir, targetBinaryName));
    console.log(`Copied ${binaryName} to dist/${targetBinaryName}`);
  }

  if (fs.existsSync(path.join(rootDir, "lantern.db"))) {
    fs.copyFileSync(path.join(rootDir, "lantern.db"), path.join(distDir, "lantern.db"));
    console.log("Copied lantern.db to dist");
  }

  console.log("\n======================================================");
  console.log("Standalone Build Success!");
  console.log("======================================================");
} else {
  console.log("[1/4] Building Vue Admin Frontend...");
  run("bun", ["install"], adminDir);
  run("bun", ["run", "build", "--", "--outDir", "../template/admin", "--emptyOutDir"], adminDir);

  console.log("[2/4] Preparing src-tauri...");
  const tauriSrcDir = path.join(tauriDir, "src");
  const tauriHandlersDir = path.join(tauriSrcDir, "handlers");

  if (!fs.existsSync(tauriHandlersDir)) {
    fs.mkdirSync(tauriHandlersDir, { recursive: true });
  }

  fs.copyFileSync(path.join(backendRustDir, "src", "models.rs"), path.join(tauriSrcDir, "models.rs"));
  fs.copyFileSync(path.join(backendRustDir, "src", "utils.rs"), path.join(tauriSrcDir, "utils.rs"));
  fs.copyFileSync(path.join(backendRustDir, "src", "db.rs"), path.join(tauriSrcDir, "db.rs"));
  copyRecursiveSync(path.join(backendRustDir, "src", "handlers"), tauriHandlersDir);

  console.log("[3/4] Building Tauri Desktop App...");
  run("bun", ["tauri", "build"], tauriDir);

  console.log("[4/4] Copying final binary...");
  const distDesktopDir = path.join(rootDir, "dist-desktop");
  if (!fs.existsSync(distDesktopDir)) {
    fs.mkdirSync(distDesktopDir, { recursive: true });
  }

  const isWindows = process.platform === "win32";
  const binaryName = isWindows ? "lantern-riddle.exe" : "lantern-riddle";
  const targetBinaryName = isWindows ? "lantern-riddle-admin.exe" : "lantern-riddle-admin";

  const releaseDir = path.join(tauriDir, "target", "release");
  const binaryPath = path.join(releaseDir, binaryName);

  if (fs.existsSync(binaryPath)) {
    fs.copyFileSync(binaryPath, path.join(distDesktopDir, targetBinaryName));
    console.log(`Copied ${binaryName} to dist-desktop/${targetBinaryName}`);
  } else {
      console.log(`Binary not found at ${binaryPath}, checking for bundled app...`);
      if (process.platform === "darwin") {
          const bundlePath = path.join(releaseDir, "bundle", "dmg");
          console.log(`On macOS, you might find the .dmg in ${bundlePath}`);
      }
  }

  if (fs.existsSync(path.join(rootDir, "lantern.db"))) {
    fs.copyFileSync(path.join(rootDir, "lantern.db"), path.join(distDesktopDir, "lantern.db"));
    console.log("Copied lantern.db to dist-desktop");
  }

  console.log("\n======================================================");
  console.log("Desktop App Build Success!");
  console.log("======================================================");
}
