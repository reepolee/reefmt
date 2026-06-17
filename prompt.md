formatter needs to convert test-in.ree to test-out.ree 
current testing output to test-out-actual.ree
make the default line wrap length 140 chars
Fix self-closing empty elements: `<tag></tag>` instead of `<tag />`
Fix expression spacing: `{= expr}` instead of `{= expr }` to match test-out.ree

readme is not curent, we are moving to rust only, no biome, ox or dprint