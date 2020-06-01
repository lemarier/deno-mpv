import { Hash, path, exists } from "../deps.ts";

export async function download(localPath: string): Promise<string | null> {
  const os = Deno.build.os;
  const md5 = new Hash("md5");

  const downloadUrl =
    "https://github.com/lemarier/libmpv/releases/download/v0.0.8";

  const FILES_MAP: { [os in typeof Deno.build.os]: string | null } = {
    darwin: "mac.zip",
    linux: null,
    windows: "win.zip",
  };

  const remoteFile = FILES_MAP[os];

  if (!remoteFile) {
    return Promise.resolve(remoteFile);
  }

  const remoteUrl = `${downloadUrl}/${remoteFile}`;
  const remoteHash = md5.digestString(remoteUrl + remoteFile).hex();

  const cacheFileName = `${remoteHash}${remoteFile}`;
  const localFile = path.join(localPath, cacheFileName);

  try {
    await Deno.mkdir(localPath, { recursive: true });
  } catch (error) {
  }

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

        break;

      default:
        break;
    }
  }

  return localPath;
}
