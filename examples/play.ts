import {
  CreateWindow,
} from "../plugin.ts";

CreateWindow(
  {
    url: Deno.args[0] ||
      "https://www.libde265.org/hevc-bitstreams/tos-1720x720-cfg01.mkv",
  },
);

console.log("window created");
