import { Hash, path, exists } from "../deps.ts";

const os = Deno.build.os;
const md5 = new Hash("md5");

const localPath = Deno.args[0] || path.resolve("mpv_source");

const downloadUrl =
  "https://github.com/lemarier/libmpv/releases/download/v0.0.8";

const FILES_MAP: { [os in typeof Deno.build.os]: string | null } = {
  darwin: "mac.zip",
  linux: null,
  windows: "win.zip",
};

async function download(): Promise<string | null> {
  const remoteFile = FILES_MAP[os];

  if (!remoteFile) {
    return Promise.resolve(remoteFile);
  }

  const remoteUrl = `${downloadUrl}/${remoteFile}`;
  const remoteHash = md5.digestString(remoteUrl + remoteFile).hex();

  const cacheFileName = `${remoteHash}${remoteFile}`;
  const localFile = path.join(localPath, cacheFileName);

  await Deno.mkdir(localPath, { recursive: true });

  if (!(await exists(localFile))) {
    const download = await fetch(remoteUrl);
    if (download.status !== 200) {
      throw Error(`downloading from "${remoteUrl}" failed.`);
    }
    const pluginFileData = await download.arrayBuffer();
    await Deno.writeFile(
      localFile,
      new Uint8Array(pluginFileData),
    );

    switch (os) {
      case "darwin":
        await Deno.run({
          cmd: [
            "unzip",
            "-o",
            localFile,
            "-d",
            localPath,
          ],
        }).status();
        break;
      case "windows":
        await Deno.run({
          cmd: [
            "7z",
            "x",
            localFile,
            `-o${localPath}`,
            "-aoa",
          ],
        }).status();

        await Deno.run({
          cmd: [
            "dir",
            path.join(localPath, "win32"),
          ],
        }).status();
        break;

      default:
        break;
    }
  }

  return localPath;
}

await download();
