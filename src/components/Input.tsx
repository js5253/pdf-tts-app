type InputProps = {
    type: string;
    name: string;
    value: string;
    // onChange: 
}
export const Input = ({type, name, value}: InputProps) =>{
    return(
        <input type={type} name={name} value={value} class="border rounded px-4 py-2 border-2 border-gray-300"/>
    )
}