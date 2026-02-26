import { createFileRoute } from "@tanstack/react-router";
import { CodeRunner } from "@/components/runner/CodeRunner";

export const Route = createFileRoute("/")({
	component: HomePage,
});

function HomePage() {
	return (
		<div className="h-full flex flex-col overflow-hidden">
			<CodeRunner />
		</div>
	);
}
