import { createEnv } from "@t3-oss/env-core";
import { z } from "zod";

export const env = createEnv({
	server: {
		DEMO_TINIRUN_URL: z.url().default("http://localhost:8082/api"),
		DEMO_TINIRUN_API_KEY: z.string(),
	},
	clientPrefix: "VITE_",
	client: {
		VITE_APP_TITLE: z.string().min(1).optional(),
	},
	runtimeEnv:
		import.meta.env.MODE === "production" ? process.env : import.meta.env,
	emptyStringAsUndefined: true,
});
