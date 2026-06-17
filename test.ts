export async function post_auth_register(req: BunRequest): Promise<Response> {
	const [translated, body_text] = await Promise.all([
		translated_from_request(req, import.meta.dir),
		req.text(),
	]);
	return translated;
}
