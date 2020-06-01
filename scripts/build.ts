export async function build(mshtml: boolean = Deno.args.includes("mshtml")) {
  const command = [
    "cargo",
    "build",
    "--release",
    "--locked",
    "--features",
    "build_libmpv",
  ];

  if (mshtml) {
    command.push("--no-default-features");
  }

  const cargo = Deno.run({
    cmd: command,
  });

  if (!(await cargo.status()).success) {
    Deno.exit(1);
  }

  const dylib = Deno.run({
    cmd: ["./scripts/fix_dylib.sh"],
  });

  if (!(await dylib.status()).success) {
    Deno.exit(1);
  }
}

if (import.meta.main) {
  await build();
}
