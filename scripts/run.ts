export async function run(
  file: string = Deno.args[0],
  mshtml: boolean = Deno.args.includes("mshtml"),
) {
  const env: {
    [key: string]: string;
  } = {
    DENO_MPV_PLUGIN_BASE: "file://./target/release",
    DENO_MPV_DEBUG: "1",
  };

  if (mshtml) {
    env["DENO_MPV_PLUGIN"] = "file://./target/release/deno_mpv.dll";
  }

  const process = Deno.run({
    cmd: ["deno", "run", "-A", "-r", "--unstable", file],
    env,
  });

  if (!(await process.status()).success) {
    Deno.exit(1);
  }
}

if (import.meta.main) {
  await run();
}
