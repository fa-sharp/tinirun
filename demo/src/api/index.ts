import { createTinirunClient } from "@tinirun/client";
import { env } from "@/env";

export const apiClient = createTinirunClient(
	env.DEMO_TINIRUN_URL,
	env.DEMO_TINIRUN_API_KEY,
);
