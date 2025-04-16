"use client";
import { CreateRoomRequest } from '@/types/CreateRoomRequest';
import { useState } from "react";
import { toast } from "sonner";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useMutation } from "@tanstack/react-query";
import { BASE_URL } from '@/const';


const RoomTypeSchema = z.enum(["presentation"]);
type RoomType = z.infer<typeof RoomTypeSchema>;





const formSchema = z.object({
  organisation_id: z.string().min(1),
  room_name: z.string().min(1),
  room_type: RoomTypeSchema,
});

// Function to create a room
const createRoom = async (data: z.infer<typeof formSchema>) => {

    const createRoomRequest: CreateRoomRequest = {
        organisation_id: data.organisation_id,
        room_type: data.room_type,
        name: data.room_name,
        slide_data: [],
    }
  const response = await fetch(`${BASE_URL}/room`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(createRoomRequest),
  });
  
  if (!response.ok) {
    throw new Error('Failed to create room');
  }
  
  return response.json();
};

export default function CreateRoomForm({onRoomCreated}: {onRoomCreated: () => void}) {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      organisation_id: "",
      room_name: "",
      room_type: "presentation" as RoomType,
    },
  });

  const mutation = useMutation({
    mutationFn: createRoom,
    onSuccess: (data) => {
      toast.success("Room created successfully!");
      console.log(data);
      form.reset();
      onRoomCreated();
    },
    onError: (error) => {
      console.error("Form submission error", error);
      toast.error("Failed to create room. Please try again.");
    },
  });

  function onSubmit(values: z.infer<typeof formSchema>) {
    mutation.mutate(values);
  }

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className="space-y-8 max-w-3xl mx-auto py-10"
      >
        <FormField
          control={form.control}
          name="organisation_id"
          render={({ field }) => (
            <FormItem>
              <FormLabel>organisation_id</FormLabel>
              <FormControl>
                <Input placeholder="r42" type="" {...field} />
              </FormControl>
              <FormDescription>organisation_id</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="room_name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>room_name</FormLabel>
              <FormControl>
                <Input placeholder="room1" type="" {...field} />
              </FormControl>
              <FormDescription>room_name</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="room_type"
          render={({ field }) => (
            <FormItem>
              <FormLabel>room_type</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
                <FormControl>
                  <SelectTrigger>
                    <SelectValue placeholder="presentation" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {Object.values(RoomTypeSchema.enum).map((value) => (
                    <SelectItem key={value} value={value}>
                      {value}
                    </SelectItem>
                  ))}{" "}
                </SelectContent>
              </Select>
              <FormDescription>room_type</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit">Submit</Button>
      </form>
    </Form>
  );
}
