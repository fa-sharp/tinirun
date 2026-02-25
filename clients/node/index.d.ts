import { Client } from "openapi-fetch";
import { paths, components } from "./types";

export type TinirunClient = Client<paths, `${string}/${string}`>;
export type TinirunSchemas = components["schemas"];

export function createTinirunClient(
  baseUrl: string,
  apiKey: string,
): TinirunClient;
