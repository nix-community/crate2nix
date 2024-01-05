import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://nix-community.github.io',
	base: '/crate2nix',
	integrations: [
		starlight({
			title: 'crate2nix - nix-community',
			social: {
				github: 'https://github.com/nix-community/crate2nix',
			},
			editLink: {
				baseUrl: 'https://github.com/nix-community/crate2nix/edit/master/docs/',
			},
			sidebar: [
				{
					label: 'Guides',
					items: [
						// Each item here is one entry in the navigation menu.
						{ label: 'Example Guide', link: '/guides/example/' },
					],
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
			],
		}),
	],
});
