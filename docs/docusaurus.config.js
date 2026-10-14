import {themes as prismThemes} from 'prism-react-renderer';

const config = {
  title: 'Swimming',
  tagline: 'Native desktop apps with Rust and React',
  url: 'https://drsounds.github.io',
  baseUrl: '/swimming/',
  organizationName: 'drsounds',
  projectName: 'swimming',
  onBrokenLinks: 'throw',
  future: {
    experimental_router: 'hash',
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.js',
        },
        blog: false,
      },
    ],
  ],
  themeConfig: {
    navbar: {
      title: 'Swimming',
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          href: 'https://github.com/drsounds/swimming',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  },
};

export default config;
