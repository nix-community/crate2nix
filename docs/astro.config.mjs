import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://nix-community.github.io',
	base: '/crate2nix',
	integrations: [
		starlight({
			title: 'crate2nix',
			social: {
				github: 'https://github.com/nix-community/crate2nix',
			},
			editLink: {
				baseUrl: 'https://github.com/nix-community/crate2nix/edit/master/docs/',
			},
			sidebar: [
				{
					label: 'Home',
					link: '/',
				},
				{
					label: 'Getting Started',
					autogenerate: { directory: '10_getting_started' },
				},
				{
					label: 'Generating',
					autogenerate: { directory: '20_generating' },
				},
				{
					label: 'Building',
					autogenerate: { directory: '30_building' },
				},
				{
					label: 'Toolchains',
					autogenerate: { directory: '35_toolchains' },
				},
				{
					label: 'External Sources',
					autogenerate: { directory: '40_external_sources' },
				},
				{
					label: 'Contributing',
					autogenerate: { directory: '50_contributing' },
				},
				{
					label: 'Design & Background',
					autogenerate: { directory: '70_design' },
				},
				{
					label: 'Reference',
					autogenerate: { directory: '90_reference' },
				},
			],
		}),
	],
});
