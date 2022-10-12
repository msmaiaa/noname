import { createApi } from "@reduxjs/toolkit/query/react";
import { apiBaseQuery } from "./baseQuery";

export const baseApi = createApi({
  baseQuery: apiBaseQuery,
  endpoints: () => ({}),
});
