import { deferred } from "./deps.ts";
import { prepare } from "./utils/load_plugin.ts";

export const PLUGIN_URL_BASE = Deno.env.get("DENO_MPV_PLUGIN_BASE") ||
  "https://github.com/lemarier/deno-mpv/releases/download/v0.0.2";
const PLUGIN_URL = Deno.env.get("DENO_MPV_PLUGIN");
const DEBUG = Boolean(Deno.env.get("DENO_MPV_DEBUG"));

let pluginId: number | null = null;

// @ts-ignore
const core = Deno.core as {
  ops: () => { [key: string]: number };
  setAsyncHandler(rid: number, handler: (response: Uint8Array) => void): void;
  dispatch(
    rid: number,
    msg: any,
    buf?: ArrayBufferView,
  ): Uint8Array | undefined;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function decode(data: Uint8Array): object {
  const text = decoder.decode(data);
  return JSON.parse(text);
}

function encode(data: object): Uint8Array {
  const text = JSON.stringify(data);
  return encoder.encode(text);
}

function getOpId(op: string): number {
  const id = core.ops()[op];

  if (!(id > 0)) {
    throw `Bad op id for ${op}`;
  }

  return id;
}

export interface MPVResponse<T> {
  err?: string;
  ok?: T;
}

/**
 * Load the plugin
 */
export async function load(cache = true, verbose = false) {
  unload();
  pluginId = await prepare({
    name: "deno_mpv",
    checkCache: cache,
    printLog: verbose,
    urls: {
      darwin: PLUGIN_URL || `${PLUGIN_URL_BASE}/libdeno_mpv.dylib`,
      windows: PLUGIN_URL || `${PLUGIN_URL_BASE}/deno_mpv.dll`,
      linux: PLUGIN_URL || `${PLUGIN_URL_BASE}/libdeno_mpv.so`,
    },
  });
}

/**
 * Free the plugin resource
 */
export function unload(): void {
  if (pluginId !== null) Deno.close(pluginId);
  pluginId = null;
}

export function CreateWindow(params?: any): any {
  return unwrapResponse(opSync("mpv_create_window", params));
}

function unwrapResponse<T, R extends MPVResponse<T>>(response: R): T {
  if (response.err) {
    throw response.err;
  }

  if (response.ok) {
    return response.ok;
  }

  throw "Invalid response";
}

function opSync<R extends MPVResponse<any>>(op: string, data: object): R {
  if (pluginId === null) {
    throw "The plugin must be initialized before use";
  }

  const opId = getOpId(op);
  const response = core.dispatch(opId, encode(data))!;

  return decode(response) as R;
}

async function opAsync<R extends MPVResponse<any>>(
  op: string,
  data: object,
): Promise<R> {
  if (pluginId === null) {
    throw "The plugin must be initialized before use";
  }

  const opId = getOpId(op);
  const promise = deferred<R>();

  core.setAsyncHandler(
    opId,
    (response) => promise.resolve(decode(response) as R),
  );

  const response = core.dispatch(opId, encode(data));
  if (response != null || response != undefined) {
    throw "Expected null response!";
  }

  return promise;
}

await load(!DEBUG, DEBUG);
window.addEventListener("unload", unload);
