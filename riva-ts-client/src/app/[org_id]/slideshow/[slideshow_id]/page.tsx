"use client";
import { CommandMessage } from "@/types/CommandMessage";
import { CommandType } from "@/types/CommandType";
import { useParams } from "next/navigation";
import { useEffect } from "react";

type Params = {
  org_id: string;
  slideshow_id: string;
};

const Page = () => {
  const { org_id, slideshow_id } = useParams<Params>();


  return (
    <div>
      <h1>Slideshow</h1>
      <h2>Slideshow ID: {slideshow_id}</h2>
    </div>
  );
};

export default Page;
