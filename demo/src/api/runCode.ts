import { createServerFn } from "@tanstack/react-start";
import { getRequest } from "@tanstack/react-start/server";
import type { TinirunSchemas } from "@tinirun/client";
import { apiClient } from ".";

/**
 * Run a code snippet and get the output stream (called on the server
 * and streamed down to the client)
 */
export const runCodeSnippetServerFn = createServerFn({ method: "POST" })
	.inputValidator((input: TinirunSchemas["CodeRunnerInput"]) => input)
	.handler((ctx) => runCodeSnippet(ctx.data));

async function runCodeSnippet(input: TinirunSchemas["CodeRunnerInput"]) {
	const abortController = new AbortController();
	getRequest().signal?.addEventListener("abort", () => {
		abortController.abort();
	});

	const res = await apiClient.POST("/code/run", {
		body: input,
		parseAs: "stream",
		signal: abortController.signal,
	});
	if (!res.response.ok) {
		throw new Error(
			`Failed to run code snippet: ${res.response.status} - ${res.error ?? "unknown error"}`,
		);
	}

	return new Response(res.data);
}
