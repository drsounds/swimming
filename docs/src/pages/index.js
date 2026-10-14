import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';

export default function Home() {
  return (
    <Layout title="Home" description="Documentation for the Swimming native UI framework">
      <main className="container margin-vert--xl">
        <h1>Swimming</h1>
        <p>
          Build native desktop apps with Rust and React, without a browser engine or WebView.
        </p>
        <Link className="button button--primary button--lg" to="/docs/intro">
          Read the documentation
        </Link>
      </main>
    </Layout>
  );
}
