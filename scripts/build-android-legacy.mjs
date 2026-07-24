import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..");
const packageJson = JSON.parse(
  readFileSync(join(repoRoot, "package.json"), "utf8"),
);
const version = packageJson.version;
const tauriCli = join(
  repoRoot,
  "node_modules",
  "@tauri-apps",
  "cli",
  "tauri.js",
);

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    env: process.env,
    stdio: "inherit",
  });

  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
}

function findAndroidSdk() {
  const candidates = [
    process.env.ANDROID_HOME,
    process.env.ANDROID_SDK_ROOT,
    process.platform === "win32" && process.env.LOCALAPPDATA
      ? join(process.env.LOCALAPPDATA, "Android", "Sdk")
      : undefined,
    process.env.HOME ? join(process.env.HOME, "Android", "Sdk") : undefined,
  ].filter(Boolean);

  return candidates.find((candidate) => existsSync(candidate));
}

function findApkSigner(androidSdk) {
  const buildToolsDir = join(androidSdk, "build-tools");
  if (!existsSync(buildToolsDir)) return undefined;

  const versions = readdirSync(buildToolsDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort((left, right) => right.localeCompare(left, undefined, { numeric: true }));

  for (const buildToolsVersion of versions) {
    const candidate = join(
      buildToolsDir,
      buildToolsVersion,
      "lib",
      "apksigner.jar",
    );
    if (existsSync(candidate)) return candidate;
  }

  return undefined;
}

function findJava() {
  if (process.env.JAVA_HOME) {
    const executable = process.platform === "win32" ? "java.exe" : "java";
    const candidate = join(process.env.JAVA_HOME, "bin", executable);
    if (existsSync(candidate)) return candidate;
  }
  return "java";
}

console.log("Building the legacy Android APK for 32-bit ARM (armeabi-v7a)...");
run("rustup", ["target", "add", "armv7-linux-androideabi"]);
if (!existsSync(tauriCli)) {
  throw new Error("Tauri CLI not found. Run npm ci before building.");
}
run(process.execPath, [
  tauriCli,
  "android",
  "build",
  "--apk",
  "--target",
  "armv7",
  "--split-per-abi",
  "--ci",
]);

const releaseDir = join(
  repoRoot,
  "src-tauri",
  "gen",
  "android",
  "app",
  "build",
  "outputs",
  "apk",
  "arm",
  "release",
);
const unsignedApk = existsSync(releaseDir)
  ? readdirSync(releaseDir)
      .filter((name) => name.endsWith("-unsigned.apk"))
      .map((name) => join(releaseDir, name))[0]
  : undefined;

if (!unsignedApk) {
  throw new Error(`No unsigned armv7 APK found under ${releaseDir}`);
}

const artifactDir = join(repoRoot, "artifacts");
mkdirSync(artifactDir, { recursive: true });

const keystorePath = process.env.ANDROID_KEYSTORE_PATH
  ? resolve(process.env.ANDROID_KEYSTORE_PATH)
  : join(repoRoot, "src-tauri", "nclientt.keystore");
const androidSdk = findAndroidSdk();
const apkSignerJar = androidSdk ? findApkSigner(androidSdk) : undefined;

if (existsSync(keystorePath) && apkSignerJar) {
  const signedApk = join(
    artifactDir,
    `NClientT-${version}-android-armeabi-v7a.apk`,
  );
  const keystorePassword = process.env.ANDROID_KEYSTORE_PASSWORD ?? "nclientt";
  const keyAlias = process.env.ANDROID_KEY_ALIAS ?? "nclientt";
  const keyPassword = process.env.ANDROID_KEY_PASSWORD ?? keystorePassword;

  const java = findJava();
  run(java, [
    "-jar",
    apkSignerJar,
    "sign",
    "--ks",
    keystorePath,
    "--ks-pass",
    `pass:${keystorePassword}`,
    "--ks-key-alias",
    keyAlias,
    "--key-pass",
    `pass:${keyPassword}`,
    "--out",
    signedApk,
    "--in",
    unsignedApk,
  ]);
  run(java, ["-jar", apkSignerJar, "verify", "--verbose", signedApk]);
  console.log(`Legacy Android APK created: ${signedApk}`);
} else {
  const copiedApk = join(
    artifactDir,
    `NClientT-${version}-android-armeabi-v7a-unsigned.apk`,
  );
  copyFileSync(unsignedApk, copiedApk);
  console.warn(
    `Signing tools or keystore not found; unsigned APK copied to ${copiedApk}`,
  );
  console.warn("Set ANDROID_KEYSTORE_PATH and Android SDK variables to sign it.");
}
