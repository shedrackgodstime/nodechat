/*
 * WHAT IS THIS FILE?
 *
 * It's the entry point for Cloudflare Pages when building for production.
 *
 * Learn more about the Cloudflare Pages integration here:
 * - https://qwik.dev/docs/deployments/cloudflare-pages/
 *
 */
import {
  createQwikCity,
  type PlatformCloudflarePages,
} from "@builder.io/qwik-city/middleware/cloudflare-pages";
import qwikCityPlan from "@qwik-city-plan";
import render from "./entry.ssr";

declare global {
  type QwikCityPlatform = PlatformCloudflarePages;
}

const fetch = createQwikCity({ render, qwikCityPlan });

const onRequest: typeof fetch = async (request, env, ctx) => {
  if (request.method !== "HEAD") {
    return fetch(request, env, ctx);
  }

  const getRequest = new Request(request.url, {
    method: "GET",
    headers: request.headers,
    redirect: request.redirect,
  });

  const response = await fetch(getRequest, env, ctx);

  return new Response(null, {
    status: response.status,
    statusText: response.statusText,
    headers: response.headers,
  });
};

export { onRequest as fetch };
