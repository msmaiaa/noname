import withAuth from "components/withAuth";
import Link from "next/link";
import React, { Component } from "react";

export const Dashboard = () => {
  return (
    <div>
      <Link href="/">Home</Link>
      <p>dashboard</p>
    </div>
  );
};

export default withAuth("admin")(Dashboard);
