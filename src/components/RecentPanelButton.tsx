import { IoPlay } from "solid-icons/io";

export const RecentPanelButton = ({ id, fileName }: RecentName) => (
    <div class="bg-gray-100 md:min-w-40 bold min-h-20 p-2 rounded flex flex-col items-between justify-between content-between"><span>{fileName}</span>
        <button class="w-20 h-20 bg-white"><IoPlay /></button>
    </div>)