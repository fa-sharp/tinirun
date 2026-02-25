import type { Client } from "openapi-fetch";
import type { components, paths } from "./types";

export type TinirunClient = Client<paths, `${string}/${string}`>;
export type TinirunSchemas = components["schemas"];

export function createTinirunClient(
	baseUrl: string,
	apiKey: string,
): TinirunClient;
