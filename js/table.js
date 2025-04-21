import { TabulatorFull as Tabulator } from "tabulator-tables";

//define some sample data
var tabledata = [];

let table = new Tabulator("#example-table", {
  maxHeight: 1000, // set height of table to enable virtual DOM
  data: tabledata, //load initial data into table
  layout: "fitColumns", //fit columns to width of table (optional)
  placeholder: "No Hits.",
  placeholderHeaderFilter: "No Hits.",
  pagination: true, //enable pagination
  paginationCounter: "rows",
  //paginationMode:"local", //enable remote pagination
  paginationSize: 500, //optional parameter to request a certain number of rows per page
  movableColumns: true,
  columnHeaderSortMulti: true,
  groupBy: [],
  groupStartOpen: [false, false],
  //responsiveLayout:"collapse",//
  columns: [
    //Define Table Columns
    { title: "Domain", field: "host", sorter: "string", headerFilter: true },
    { title: "Title", field: "title", sorter: "string", headerFilter: true },
    {
      title: "URL",
      field: "url",
      headerFilter: true,
      sorter: "string",
      formatter: "link",
      formatterParams: {
        labelField: "",
        urlPrefix: "",
        target: "_blank",
      },
    },
    {
      title: "Description",
      field: "auto_descr",
      sorter: "string",
      headerFilter: true,
      tooltip: function (e, cell, onRendered) {
        return cell.getRow().getData().auto_descr;
      },
    },
  ],
  printRowRange: "active",
  printAsHtml: true,
});

// behaviour when the row is clicked
table.on("rowClick", async function (e) {
  console.log("received row click event - doing nothing with it");
});
