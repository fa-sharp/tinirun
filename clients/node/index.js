import createClient from "openapi-fetch";

export const createTinirunClient = (baseUrl, apiKey) =>
  createClient({
    baseUrl,
    headers: {
      "X-Runner-Api-Key": apiKey,
    },
  });
