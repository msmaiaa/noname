import type { NextPage } from "next";
import Link from "next/link";

const Home: NextPage = () => {
  return (
    <div>
      <p>This is the home page</p>
      <Link href="/admin/dashboard">admin panel</Link>
    </div>
  );
};

export default Home;
