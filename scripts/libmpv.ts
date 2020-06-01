import { path } from "../deps.ts";
import { download } from "../utils/libmpv.ts";

await download(Deno.args[0] || path.resolve("mpv_source"));
