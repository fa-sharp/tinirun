import { createServerFn } from "@tanstack/react-start";
import type { TinirunSchemas } from "@tinirun/client";
import { EventSourceParserStream } from "eventsource-parser/stream";
import { apiClient } from ".";

/**
 * Run a code snippet and get the output stream (called on the server
 * and streamed down to the client)
 */
export const runCodeSnippetServerFn = createServerFn()
	.inputValidator((input: TinirunSchemas["CodeRunnerInput"]) => input)
	.handler(async function* (ctx) {
		const stream = await runCodeSnippet(ctx.data);
		for await (const chunk of stream) {
			yield chunk;
		}
	});

async function runCodeSnippet(
	input: TinirunSchemas["CodeRunnerInput"],
): Promise<ReadableStream<TinirunSchemas["CodeRunnerChunk"]>> {
	const res = await apiClient.POST("/code/run", {
		body: input,
		parseAs: "stream",
	});
	if (!res.response.ok || !res.data) {
		const error = await res.response.text();
		throw new Error(
			`Failed to run code snippet: ${res.response.status} - ${error}`,
		);
	}

	return res.data
		.pipeThrough(new TextDecoderStream())
		.pipeThrough(new EventSourceParserStream())
		.pipeThrough(
			new TransformStream({
				transform(chunk, controller) {
					try {
						controller.enqueue(JSON.parse(chunk.data));
					} catch (error) {
						console.error("Error parsing stream chunk:", { chunk, error });
					}
				},
			}),
		);
}
