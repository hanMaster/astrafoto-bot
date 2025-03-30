// async fn _maybe_need_ask(&mut self) {
// let mut orders_to_remove = vec![];
// for mut o in self.orders.iter_mut() {
//     if !o.images.is_empty() && o.state.eq("new") {
//         self.send_message(o.chat_id.clone(), self.paper_prompt())
//             .await;
//         o.state = "paper_requested";
//         o.iter_count = 1;
//         o.last_update_time = SystemTime::now();
//     } else if !o.images.is_empty()
//         && (o.state.eq("paper_requested")
//             || o.state.eq("size_requested")
//             || o.state.eq("size_selected"))
//     {
//         if o.iter_count < 4 && o.last_update_time.elapsed().unwrap().as_secs() > 30 {
//             o.iter_count += 1;
//             o.last_update_time = SystemTime::now();
//
//             let msg = if o.state.eq("paper_requested") {
//                 self.paper_prompt()
//             } else if o.state.eq("size_requested") {
//                 self.size_prompt(&o.paper)
//             } else {
//                 "READY".to_string()
//             };
//
//             self.send_message(o.chat_id.clone(), msg).await;
//         } else if o.iter_count < 4 && o.last_update_time.elapsed().unwrap().as_secs() < 30 {
//         } else {
//             orders_to_remove.push(o.chat_id.clone());
//         }
//     } else if o.images.is_empty() {
//         // Delete order without images after 1 minute
//         if o.last_update_time.elapsed().unwrap().as_secs() > 60 {
//             orders_to_remove.push(o.chat_id.clone());
//         }
//     }
// }
// for chat_id in orders_to_remove {
//     self.orders.remove(&chat_id);
//     self.send_message(
//         chat_id.clone(),
//         "Заказ отменен, из-за длительного ожидания".to_string(),
//     )
//     .await;
// }
