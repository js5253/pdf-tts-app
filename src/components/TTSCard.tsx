import { IoFolder, IoFolderOpen, IoFolderOpenOutline } from "solid-icons/io"

export const TTSCard = ({item}) => {
    return(
        <div  class="flex items-center justify-between flex-row bg-white w-full p-4 border border-gray-300 my-4 border-box">
            <div>
                            <h3 class="text-lg font-bold">{item}</h3>

            </div>
            <div>
                <IoFolderOpenOutline size={25} />
            </div>
            </div>
    )
}