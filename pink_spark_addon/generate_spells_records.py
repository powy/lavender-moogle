import pandas as pd


def excel_to_lua(excel_file, sheet_name):
    # Read the Excel file
    df = pd.read_excel(excel_file, sheet_name=sheet_name)

    # Get the column names from the first row
    columns = df.columns

    # Generate Lua code
    lua_code = "local records = {\n"
    for index, row in df.iterrows():
        lua_code += f"    ['{row['name']}'] = {{ "
        field_entries = []
        for col in columns:
            value = row[col]
            if isinstance(value, str):
                field_entries.append(f"{col} = '{value}'")
            else:
                field_entries.append(f"{col} = {value}")
        lua_code += ", ".join(field_entries)
        lua_code += " },\n"
    lua_code += "}\n\nreturn records"

    return lua_code


# Define the input and output files
excel_file = "spell_list.xlsx"
sheet_name = "spells"
lua_output_file = "output.lua"

# Generate Lua code and write to file
lua_code = excel_to_lua(excel_file, sheet_name)
with open(lua_output_file, "w") as file:
    file.write(lua_code)

print(f"Lua code has been generated and written to {lua_output_file}")
