import { userStore } from "store/user";
import LoginButton from "./LoginButton";

const Navbar = () => {
  const loggedIn = userStore((state) => state.loggedIn);
  return <div>{!loggedIn && <LoginButton />}</div>;
};

export default Navbar;
